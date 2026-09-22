# 06 — Rule And Graph Logic Contracts

Status: `PLANNED — CURRENT LOGIC PACKAGE DOES NOT COMPILE`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

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

Existing `v3_logic_recursion.rs`, graph, witness, hellgate, and mutation suites
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
   HIR and algebra edges and add sema
9. `/Cargo.lock` — integrator only if the edge cut changes it
10. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
11. the exact logic and pipeline rule/graph tests listed below
12. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/06.md`
13. `crates/quanta-gqlang-v2-logic/tests/support/mod.rs` when repeated `sp`,
    `id`, and graph-reference helpers from touched tests are consolidated
14. `tools/architecture/check_architecture.py` and its `valid`/transition
    fixtures — integrator only; move logic→algebra from final to declared
    transitional edges while it is still present, then remove the actual and
    transitional edge in this ticket; rerun the architecture tests and both
    transition/final modes. Ticket 01's earlier edge-count receipt is historical
    and does not prove this changed graph.

`crates/quanta-gqlang-v2-sema/src/rule.rs` is a read-only input owned and frozen
by ticket 04. If it lacks resolved IDs, polarity, spans, or budget projection,
stop and amend 04; do not patch sema from this ticket.

## Rule Implementation

1. Accept only the complete Rule enumeration sealed by ticket 04's
   `SemanticDocumentV2`, not an arbitrary caller-supplied
   `Vec<ResolvedRuleV2>`; logic performs no lexical name, variable,
   call-kind, or arity resolution.
2. Validate that every dependency target ID belongs to the frozen program and
   that every edge retains polarity/span.
3. Build one document rule program before Tarjan/stratification.
4. Change the existing dependency graph/Tarjan inputs from names to stable
   IDs. Only locally declared predicate targets become SCC nodes/edges;
builtin/external calls are not silently inserted as local graph nodes.
   Ticket 04's `ResolvedRuleV2::calls()` contains only source-local predicate
   dependencies; builtin/host call facts remain in the typed body but are not
   SCC edges. A bare types `Predicate(id)` target does not prove local origin.
5. Reject recursive aggregate and negation cycles from the retained edge
   polarity.
6. Preserve head/body/edge checked locations on deterministic cycle
   diagnostics; choose a canonical offending edge and actual cycle path
   independent of source/call insertion order.
7. Migrate the existing recursive-aggregate matrix carrier to fixed
   `RejectAllRecursiveAggregatesV2` policy. Remove default `min`/`max`
   admission and public `with_admitted` extension from the V2 path; this packet
   has no configurable recursive aggregate exception.

## Graph Implementation

1. Keep waypoint constraints as path predicates only.
2. Require explicit bounded depth for every transitive traversal in this packet.
3. Preserve max path length, witness cap, dedup identity, tie-break, relation
   shape, and endpoint contract in `GraphQueryContractV2`.
4. Use iterative graph traversal or a checked graph-depth budget. This
   frontend performs static contract validation only; it does not enumerate
   runtime witnesses.
5. Preserve nested FIND contracts inside CHECK.

## Required Contracts

```rust
// Inputs are the sealed Ticket 04 `ResolvedRuleV2` and `ResolvedRuleCallV2`.
// Each call retains its exact SourceQualifiedNodeIdV2, types-owned target,
// polarity, and CheckedLocatedSpanV2; logic does not define a second edge DTO.
pub struct RuleProgramContractV2 {
    document_source_id: SourceIdV2,
    member_item_ids: Vec<SemanticItemIdV2>,
    rules: Vec<ResolvedRuleV2>,
    proof: StratificationProofV2,
}
```

The Ticket 04 producer must compare each expected call-node ID, resolved
target, polarity, and checked location to its types/HIR facts. A right call ID
with a transplanted target is invalid. Logic projects only source-predicate
targets into SCC edges; builtin or external-host calls retain their semantic
facts but are not local recursive dependencies. Mutation tests cover both
target transplant and omitted/duplicate/polarity-flipped call edges.
The logic adapter reads the exact Rule item IDs from the same sealed semantic
document, sorts/compares them to the program members, and rejects duplicate,
missing, extra, foreign-source, or nonlocal-target IDs. The `Vec` field above
is stored output, not a caller-selected proof input.

`GraphQueryContractV2` must expose exact witness bounds through read-only
accessors. A Boolean `bounded=true` is insufficient.

## Step-By-Step Procedure

1. Verify ticket 04 produced predicate IDs and predeclared every source rule
   name; reopen 04 if the frozen input contract is incomplete.
2. Validate and consume ticket 04's resolved dependency edges and complete
   Rule item enumeration; do not rewalk expression HIR or re-resolve calls.
3. Construct one `RuleProgramContractV2` from all document rules.
4. Change Tarjan/stratification inputs from names to stable predicate IDs.
5. Preserve user-facing names only for deterministic diagnostics/rendering.
6. Reject recursive aggregate and negation edges before producing a success
   proof.
7. Remove `lower_source_rule`, `root_symbol_name`, and call-argument dependency
   heuristics.
8. Require explicit bounded depth for transitive graph traversal; no acyclic
   proof shortcut is admitted in this packet.
9. Retain via/through as ordered path constraints.
10. Replace recursive `TarjanState::strong_connect` with iterative frames so
    the admitted 50k-node graph cannot exhaust the process stack. Charge
    each edge visit, SCC emission, and stratification relaxation before work
    or allocation; test exact limit and one-under step budget.
11. Return exact graph and rule contracts to pipeline/planning.

The private logic counter receives `max_rule_graph_nodes`,
`max_rule_graph_edges`, `max_rule_steps`, `max_witness_depth`, and
`max_witnesses` from the immutable frontend budget. It charges each source
contract node/edge and each SCC/stratification/static-validation visit before
work/allocation. It validates declared depth and witness caps against the
budget but does not execute graph traversal or emit witnesses. Exact limit
succeeds; the next charge rejects.
No surface, sema, algebra, or planning counter is reused or recharged here.

## Test Files

1. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs` — document program, ID-based SCC, polarity, spans
2. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs` — resolved-edge adapter and no raw-HIR/name fallback
3. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs` — negative/aggregate cycles and invalid targets
4. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_depth_waypoint_conflict.rs` — counterexamples and explicit bounds
5. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs` — graph budget exhaustion
6. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs` and `crates/quanta-gqlang-v2-logic/tests/test_endpoint_conflict.rs` — exact retained contract fields
7. `[MOVE+REWRITE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_punchlist.rs` to `crates/quanta-gqlang-v2-logic/tests/v3_logic_mutation_rails.rs` — mutation cases without a second suite
8. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_source_rule_document.rs`
9. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_graph_contract_retention.rs`

Existing SCC/stratification/witness positives remain in their owner files. The
two new pipeline files prove public document reachability only.

The existing 48 logic tests are a migration inventory, not 48 new tests:
keep Tarjan positive/mutual/self-loop and stratification cases in
`v3_logic_recursion.rs` but replace string keys and raw-HIR `from_rule`;
replace the old `min` recursion admission and `with_admitted` extension with
the fixed reject-all policy. In `v3_logic_hellgate.rs`, keep negation and
aggregate cycles but assert the exact canonical edge-span diagnostic;
unknown quantifier negatives move to sema. In `v3_graph_query_hellgate.rs`,
flip the old PointsTo-without-depth and via-without-depth positives to exact
bound failures. In `v3_graph_query_depth_waypoint_conflict.rs`, depth together
with via/through must become valid and retain waypoint order. In
`v3_logic_witness.rs`, keep cap/dedup/tie-break fields but remove the old
acyclic-without-depth positive and keyword-only pins. `v3_logic_source.rs`
EXISTS-to-Semi/Anti belongs to Ticket 05, not logic. Rewrite
`test_endpoint_conflict.rs` to assert exact code, role, and checked location,
not any error. External integration tests build source documents through
Ticket 04's public sema transaction; they must not make private sema
constructors public or invent a logic-side test DTO.

Before the final DAG check, migrate every logic public diagnostic/request
field that currently names HIR `SpanV1` or `GraphQueryKindV1` to Ticket 04's
checked location and sema-reexported closed graph-kind carrier (or a single
sema-owned wrapper). The direct HIR edge appears in
`logic/src/diagnostic.rs`, `graph.rs`, and `witness.rs`; removing only the
Cargo line is not a migration. The inherited `logic/src/source.rs` imports
algebra solely to construct Semi/Anti joins. Transfer that behavior and its
tests to Ticket 05, remove `source.rs`'s algebra use and the logic→algebra
Cargo edge, and keep Rule/Graph typed adaptation within logic. This is a
disjoint 05/06 owner handoff: Ticket 05 edits algebra, Ticket 06 edits logic,
and Ticket 07 alone reconnects the pipeline; neither owner edits the other's
crate to make a focused suite pass.

`recursion.rs` currently inserts every call target as a graph node, including
an undeclared target, and its recursive Tarjan DFS has no budget. The typed
adapter must fail closed on a nonlocal predicate target, omit builtin/host
calls from local SCC edges, use an explicit iterative work stack, and charge
all relaxation visits. A sorted list of SCC names is not a diagnostic cycle
witness: select a canonical offending negated/aggregate edge and prove an
actual return path in the SCC, then pin edge location and permutation-stable
rendering.

Audited existing-test disposition:

1. Strengthen `test_endpoint_conflict.rs` to assert
   `logic.graph_endpoint_unexpected`, endpoint role, and exact span; never
   replace the assertion with printing.
2. Add an exact `logic.non_monotone_recursion.negated_recursion` fixture.
   **Amended at W3 dispatch (2026-09-21, integrator):** this wording is
   superseded. The negated arm is deliberately unreachable — a rule that
   negates its own recursion is a negation cycle and rejects as
   `logic.negation_cycle` before the non-monotone check runs (see the
   `NonMonotoneRecursion` doc comment in `logic/src/diagnostic.rs` and the
   precedence rail migrated from `v3_logic_hellgate.rs`). No
   `.negated_recursion` code exists or will be added; the required proof is
   the precedence rail (negated case lands in `logic.negation_cycle`, not in
   `logic.non_monotone_recursion.*`), which W3-A owns with an exact
   canonical-edge + return-path payload.
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
| `rule r : x if x = 1 and r(x)` | anchored recursive positive SCC |
| `rule r : x if x = 1 and not r(x)` | anchored `logic.negation_cycle` |
| `rule r : x if x = 1 and count(r(x)) > 0` | anchored recursive aggregate reject |
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
cargo clippy -p quanta-gqlang-v2-logic --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

`v3_source_rule_document` and `v3_graph_contract_retention` are deferred to
ticket 07 and enumerated in receipt 06.

## Forbidden Shortcuts

1. extracting dependencies from symbol names or callee text
2. stratifying one source rule at a time
3. treating predicate call arguments as predicate dependencies
4. downgrading unsupported calls to nonrecursive success
5. using waypoint count as any termination bound
6. recomputing graph contracts in planning

## RED

```text
rule r : x if x = 1 and r(x)             -> recursive=false
rule r : x if x = 1 and not r(x)         -> recursive=false
rule r : x if x = 1 and count(r(x)) > 0  -> recursive aggregate accepted
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

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes conflicting earlier wording.

### Owner boundary and focused proof

- Ticket 06 may reach only `FOCUSED_GREEN`; compile/test `logic` only. Pipeline
  graph/rule proof is ticket 07 integration work.
- Logic consumes sema-resolved identities and Bool proof. It does not resolve
  callee strings, infer types, validate aggregate placement, or perform runtime
  graph traversal.
- Rename `tests/v3_logic_punchlist.rs` to
  `tests/v3_logic_mutation_rails.rs` in this ticket. Ticket 09 therefore has no
  surviving `_punchlist` exception.

### Exact rule and graph contracts

- Reuse ticket 03 `ResolvedPredicateIdV2`; `PredicateIdV2` is forbidden.
- Reuse ticket 04 `TypedBoolExprV2` for bodies. Unary rules use one
  `head_term: TypedExprV2`, never `Vec<TypedExprV2>`.
- `ResolvedRuleV2`, `ResolvedPredicateEdgeV2`, `RuleProgramContractV2`, and
  `GraphQueryContractV2` have private fields and owner-only fallible
  constructors.
- Every result retains `SemanticItemIdV2`. A document-level
  `RuleProgramContractV2` additionally retains the document source identity and
  exact sorted member-item IDs. Duplicate, missing, extra, or foreign IDs
  reject.
- `GraphQueryContractV2` is produced from the exact `ResolvedGraphQueryV2` and
  is the only graph value supplied to a CHECK contract in ticket 07.
- `ResolvedRuleV2` and `RuleCallPolarityV2` are sema-produced ticket 04
  contracts. The earlier sketch's logic-owned `BodyPolarityV1` in a sema
  output is invalid under the dependency DAG; logic maps the already resolved
  polarity into its internal graph edge. It never asks HIR for a call name.
- Unanchored recursive rules such as `rule r : x if r(x)` are rejected by the
  ticket 03 types-owned singleton constraint proof before this stage. The
  positive/negated/aggregate source fixtures above include the `x = 1`
  concrete anchor and must first pass ticket 04 owner typing; logic then tests
  their polarity and SCC result, not a guessed variable type.

### Static boundedness only

This repository charges source-contract nodes/edges, SCC/stratification work,
and static validation walks. It does not execute a graph or emit witnesses.
Source `depth` must be positive and `<= max_witness_depth`; the effective
declared witness cap must be positive and `<= max_witnesses`. Exact-limit and
one-over cases are mandatory. Remove any independent
`DEFAULT_MAX_WITNESSES`; defaults come only from `FrontendBudgetV2`.

No unowned "acyclic proof" shortcut is accepted in this packet. Every graph
shape that can traverse transitively requires an explicit bounded depth. If a
future proof type is desired, it needs a separately owned producer and packet.

Required tests include multi-cycle and permuted-edge canonical SCC witnesses,
positive acyclic diamond, negation-cycle rejection, unary-head construction,
foreign/duplicate item identities, depth/witness exact boundary and one-over,
and source permutation stability.

Requires 04. May run in parallel with 05 after sema contracts are frozen.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954`)

Ticket 06 is not implemented. Existing `recursion.rs` already owns Tarjan and
stratification, while `graph.rs`/`witness.rs` own endpoint and witness
classification; none is a candidate for a second V2 algorithm. The inherited
`GraphQueryContractV1` carries only kind, witness, and span, and the old
`PointsTo` acyclic-by-treatment shortcut conflicts with this packet's explicit
bound policy. Migrate those owners in place.

The first admissible input is Ticket 04's complete document rule set with
resolved predicate IDs, exact polarity/spans, unary typed heads, and graph
selector identities. `logic/src/source.rs` must become a typed adapter only:
no raw HIR, name lookup, or call-argument dependency fallback. The owner test
must distinguish an anchored positive self-loop from unanchored type failure,
negated loop, aggregate loop, mutual SCC, and a missing target ID. For graph,
assert every transitive subkind's depth/cap at exact limit and one over,
waypoints insufficient without depth, and exact resolved endpoint/witness
fields. Strict logic owner proof precedes the Ticket 07 public-document
reachability test; a preexisting Tarjan unit pass is not source reachability.

Additional producer/consumer gate: an earlier Ticket 04 draft exposed public
rule/call constructors with only same-source validation. The later sema
43/43 owner checkpoint has source-local self/mutual Rule call positives with
call-node IDs, but its complete authored rule/call-set and intrinsic
target/polarity equality still lack mutation proof. Ticket 04 must seal the
HIR/types call set before Ticket 06 starts. Ticket 06 then supplies mutation fixtures
for omitted, duplicated, foreign-rule, target-transplanted, and
polarity-flipped edges and proves SCC/negation/aggregate classification
changes or rejects deterministically. A name-based rebuild in logic is not a
repair.

The Ticket 04 producer must classify every local source-call dependency from
the exact resolved call and intrinsic ancestor aggregate effects, not the
composed effect of a scalar wrapper that contains an aggregate child. Logic
consumes the sealed polarity and rejects a forged polarity/call-path witness;
it does not reinterpret callable names or repair sema's traversal. Include a
source fixture where a scalar outer call has an aggregate child and a
separate source-call argument, proving the local edge is not spuriously
`Aggregated`, plus a true aggregate-over-source-call rejection fixture.

The latest Ticket 04 sema 52/52 owner suite now contains the same-scalar-call
source polarity oracle: `is_null({aggregate: count(1), recursive: r(x)})`
keeps `r(x)` Positive, while `count(r(x))` marks it Aggregated. This is
producer-local evidence only. Ticket 06 still must consume the exact sealed
call graph, reject an aggregate recursive SCC, and prove its own mutation
for omitted/foreign/transplanted/polarity-flipped edges; no logic integration
or final architecture edge removal is implied.

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: `PLANNED`, start gate not met. Logic 49/49 is green, but those tests
exercise V1 adapters over raw HIR/name strings and therefore prove reusable
Tarjan/stratification/graph algorithms, not the Ticket 06 contract.

Single implementation path:

1. Accept only Ticket 04's complete `ResolvedRuleV2` set and resolved graph
   contracts. Delete `RuleDefV1::from_rule`, raw `RuleHirV1` dependency
   collection, `GraphQuerySemanticsV1::classify(&GraphQueryHirV1)`, and name-
   based target reconstruction from the public production path.
2. Verify exact rule membership, target identity, polarity, call-node ID, and
   call location before building the program graph. Reject missing, duplicate,
   foreign-source, target-transplanted, and polarity-flipped facts.
3. Run the existing Tarjan/stratification owner once over the full document
   program. Preserve canonical SCC/cycle witnesses under edge permutation;
   reject negative and aggregate recursion, accept anchored positive recursion.
4. Consume sealed graph endpoint/selector IDs and enforce explicit positive
   depth plus witness cap for every transitive subkind. Remove the inherited
   `PointsTo` acyclic shortcut and independent default witness cap.
5. Move relational EXISTS lowering to Ticket 05, delete `logic/src/source.rs`
   production use and `quanta-gqlang-v2-algebra` from Logic's manifest.
6. Update architecture fixtures and final allowlist in the same cut so Logic
   ends with only Sema + Types normal dependencies. No dangling transition
   exception is allowed.

Required tests: positive self/mutual SCC, negation cycle, true aggregate cycle,
scalar-wrapper sibling polarity, host/builtin exclusion, canonical multi-cycle
witness, acyclic diamond, every transitive graph subkind at depth/cap limit and
one over, and the five rule-fact mutation classes above.

Acceptance requires Logic all-target tests, strict Clippy, architecture unit
fixtures, and the live architecture checker. The existing 49/49 count may be
retained only for migrated assertions; it is not a receipt by itself.

## Final Independent RR — Algorithm Reuse Boundary

No second SCC, stratifier, witness selector, or graph resolver is authorized.
The existing 49/49 suite proves reusable algorithms only. Begin migration after
Ticket 04 supplies exact source Rule edges and disjoint host/Composition graph
targets. The final owner tests must demonstrate that builtin and host calls are
excluded from the source-rule dependency graph, and that local graph targets
are consumed without name lookup or host fallback. Ticket 06 owns the final
Logic→Algebra edge deletion; Ticket 01 only verifies the resulting graph.
## 2026-09-21 Final Current-Tree Audit

Status remains `PLANNED`. Logic's direct owner suite is 49/49 GREEN. That proves
the existing Tarjan/stratification/graph algorithms on their own inputs, not
source-language reachability or document-wide assembly.

Reuse the algorithms. Do not add a second SCC engine or infer edges from names.
Remaining work:

1. Consume exact Sema Rule identities/call targets and build one document-wide
   program in authored order.
2. Preserve positive/negative/aggregate polarity from the sealed Rule calls;
   host and builtin calls must never become local SCC vertices.
3. Consume typed graph contracts and bounded witness policies without reading
   HIR. Delete Logic's transitional HIR edge after migration.
4. Lower Query EXISTS only from Ticket 05's checked semi/anti algebra; retain
   Rule-body EXISTS as the explicit typed rejection pinned by Ticket 04.
5. Add source-level mutual recursion, negation cycle, recursive aggregate,
   host/source same-spelling, graph bound exact/+1, permutation, and identity
   transplant tests.
6. Record both algorithm-owner and source-adapter counts/hashes in receipt 06.

Passing 49 tests is not a substitute for the source adapter or receipt.

## SS-06A Current-Tree Logic Audit — 2026-09-21

Status remains `PLANNED`. The reusable Tarjan/stratification vocabulary exists,
and Ticket 04 already exposes the required sealed producer inputs:
`ResolvedRuleV2` retains predicate/call target/polarity/location and
`SemanticDocumentV2::rules()` supplies one closed document-wide set. FIND also
already retains resolved selectors and immutable witness bounds. Do not reopen
Ticket 04 to recreate those facts.

Logic itself is still V1: it has no Sema dependency, accepts raw HIR/name/span
inputs, reconstructs dependencies by expression traversal, and Pipeline feeds
one-rule programs. As a result, source-level mutual recursion is unreachable.
The current graph classifier also treats waypoint count as a depth substitute,
owns an independent default witness cap, and permits external recursive
aggregate admission. `ExistenceCheckV1` independently manufactures algebra
Semi/Anti joins and must move to Ticket 05 rather than survive as a second
producer.

The old `49/49` claim is not a current qualification receipt: a targeted Logic
test/list attempt timed out before enumeration, and strict Logic Clippy is
blocked by Algebra's retained `dead_code` compatibility annotations. Required
replacement path: one `RuleProgramContractV2::from_document` factory;
iterative budgeted SCC traversal; fixed reject-all recursive aggregate policy;
resolved graph adapter retaining explicit depth/ordered waypoints; then delete
raw HIR/Algebra edges and V1 fixture suites in the same cut. Record a fresh
owner result only after that replacement compiles.

## SS-06B Final Current-State Supersession — 2026-09-21

This section supersedes all `49/49 GREEN`, timeout-only, and Clippy-blocked
current-state statements above. Historical V1 algorithm results remain useful
only as migration inventory. Audited tree: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the shared dirty overlay.

### Current disposition

| Area | Disposition | Current fact |
|---|---|---|
| Logic package build | `RED` | fresh `cargo check -p quanta-gqlang-v2-logic --all-targets` fails in `src/source.rs` on the deleted Algebra V1 imports |
| Document-wide Sema adapter | `NOT IMPLEMENTED` | Logic has no Sema dependency and no `RuleProgramContractV2::from_document` |
| Existing Tarjan/stratifier | `REUSABLE ALGORITHM, NOT QUALIFIED` | name-keyed, recursive DFS, unbudgeted, and callable only through V1 caller-built rules |
| Rule membership/edge validation | `NOT IMPLEMENTED` | no exact document member set, predicate ID, call-node ID, checked location, or transplant validation |
| Recursive aggregate policy | `WRONG FOR PACKET` | V1 defaults admit `min`/`max` and expose `with_admitted`; this packet requires reject-all recursion |
| Graph adapter | `WRONG INPUT/POLICY` | consumes raw `GraphQueryHirV1`, treats waypoint count as a path bound, rejects depth+waypoint, grants PointsTo an acyclic shortcut, and invents cap 64 |
| Relational EXISTS ownership | `DUPLICATE AND BROKEN` | `logic/src/source.rs` still constructs retired Algebra V1 Semi/Anti nodes; Ticket 05 is the sole owner now |
| HIR/Algebra dependency cut | `NOT IMPLEMENTED` | manifest still depends on Algebra + HIR + Types and has no Sema edge |
| Owner tests/Clippy | `NOT RUNNABLE` | compilation fails before test enumeration; there is no current Logic GREEN count |

The direct compiler error is not a reason to restore aliases. `source.rs`
imports `AlgebraNodeV1`, `AlgebraOpV1`, `SemiJoinV1`, and `AntiJoinV1`, all
intentionally deleted by SS-05B. Remove that second EXISTS producer and migrate
its semantic cases to Ticket 05 owner tests.

### Exact implementation sequence

1. Add the Sema dependency and implement one
   `RuleProgramContractV2::from_document(&SemanticDocumentV2)` entry. Read the
   rule graph and witness limits from the document's SS-04W immutable budget
   receipt. Enumerate the complete Rule item set internally; do not accept a
   caller-selected `Vec` or a separate budget.
2. Project each `ResolvedRuleV2` to stable predicate IDs and exact
   `ResolvedRuleCallV2` edges. Validate member item IDs, call IDs, targets,
   polarity, source/registry identity, and checked location before allocating
   the graph. Reject missing, duplicate, extra, foreign, target-transplanted,
   and polarity-flipped facts.
3. Reuse the SCC/stratification algorithm but replace string vertices with
   stable IDs and recursive `strong_connect` with explicit iterative frames.
   Charge nodes, edges, frame pushes, SCC emission, cycle-witness search, and
   every stratum relaxation before work/allocation.
4. Treat only predicates in the sealed document member set as SCC vertices.
   Builtin/host calls remain typed body facts and never become local graph
   nodes. An unresolved/nonlocal predicate target fails closed rather than
   silently creating a vertex.
5. For a negated or aggregate recursive edge, choose a canonical offending
   edge and prove an actual return path. Preserve its checked location and make
   the rendered witness invariant under rule/edge insertion permutation.
6. Replace the configurable recursive aggregate matrix with one V2 reject-all
   policy for recursive SCCs. Nonrecursive aggregate legality remains the Sema
   contract; Logic performs no spelling-based exception.
7. Adapt `ResolvedGraphQueryV2`, not HIR. Enforce required/extraneous endpoint
   roles, an explicit positive depth for every transitive family, and the exact
   `GraphWitnessBoundsV2` depth/cap. Retain ordered via/through constraints;
   depth plus waypoints is valid and waypoints never substitute for depth.
8. Delete `logic/src/source.rs` production exports, the Logic→Algebra and
   Logic→HIR manifest edges, raw span/request DTOs, independent default witness
   cap, and V1 caller-configurable aggregate admission in the same cut. Final
   normal dependencies are Sema + Types only.
9. Update architecture fixtures/allowlists with the real edge deletion and
   reconnect Pipeline only in Ticket 07. Do not make Pipeline compile through a
   temporary V1 adapter.

Sema currently supplies the needed Rule set, exact rule-call identity/polarity/
location, resolved graph references, explicit depth, direction, and immutable
witness bounds. It does **not** yet retain the rule node/edge/step budget on the
final document; SS-04W owns that small producer repair so Logic never accepts a
caller-selected replacement. `GraphQueryKindV1` is re-exported through Sema,
so Logic need not retain a direct HIR dependency; a Sema-owned V2 wrapper may
replace the re-export, but a Logic-local duplicate enum is forbidden.

### Required owner/mutation matrix

- positive self recursion, mutual recursion, nonrecursive dependency, and
  acyclic diamond over stable IDs;
- negation and aggregate cycles with exact canonical offending edge/location
  and actual return path, invariant under rule/edge permutation;
- missing/duplicate/extra/foreign member, nonlocal target, transplanted target,
  wrong call ID, wrong location registry, and polarity flip;
- builtin/host same-spelling exclusions and source predicate/host collision;
- iterative SCC at exact node/edge/step limits and +1, including a 50k-deep
  chain without process-stack recursion;
- every transitive FIND subkind with explicit depth/cap exact limit and +1;
  PointsTo follows the pinned packet rule rather than a hidden acyclic bypass;
- ordered via/through retention, depth+waypoint positive, missing/zero depth,
  missing/zero witness cap, unexpected endpoint, and nested CHECK/FIND;
- every recursive aggregate spelling rejects; nonrecursive aggregate does not
  gain a second Logic legality decision;
- compile-time/architecture assertions that no Logic production source names
  HIR or Algebra.

### Next SS/RR acceptance

SS-06B begins after SS-05C fixes the sole EXISTS lowering path. Its receipt is
accepted only when these pass on one unchanged source fingerprint:

```bash
cargo test -p quanta-gqlang-v2-logic --all-targets
cargo clippy -p quanta-gqlang-v2-logic --all-targets -- -D warnings
cargo test -p quanta-gqlang-v2-architecture-tests --all-targets
python3 tools/architecture/check_architecture.py
cargo tree -p quanta-gqlang-v2-logic --edges normal
cargo fmt --check -p quanta-gqlang-v2-logic
git diff --check -- crates/quanta-gqlang-v2-logic \
  docs/plans/sep-21-gqlang-sota-frontend-reconstruction/tickets/06-rule-graph-logic-contracts.md \
  docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/06.md
```

RR-06B additionally inspects that the live package graph contains Sema + Types
only, the 50k/depth budget cases actually run, and Pipeline compiles only after
its Ticket 07 migration. No previous `49/49` number may be carried into the new
receipt without fresh execution of migrated assertions.
