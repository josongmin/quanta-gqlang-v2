# Worker Start Here

## Current Override — 2026-09-23

Freeze before every work package. Before this documentation update, `main`
and `origin/main` were clean at
`d9d0f28a8365e6918dc8c7d0c552a8e277a838f0`. W1-W6 code is integrated;
the `ecc38f59` dirty-overlay order below is historical and **must not be
redispatched**. The pre-commit `just ci` passed 1,168/1,168, but no clean
`d9d0f28` qualification receipt exists. Read Ticket 09's current-source
section for the remaining owner-oracle audit and source-pin qualification.

Current work order:

```text
Audit 18-root source-to-checked semantics against existing owner tests
  -> add only a demonstrated missing assertion or source-level fix
  -> just ci once on final source/test/tool content
  -> clean committed-HEAD just qualify-local
  -> docs/receipt-only attestation and separate Semantica paired proof
```

### Historical worker dispatch below — 2026-09-21

The following SS-02G through SS-09B lane plan records how the original
reconstruction was executed. It is not current implementation authority.

Historical implementation order:

```text
SS-02G inventory/receipt prep || SS-03H owner tests || SS-03H-S + SS-04W Sema owner
  -> SS-05C
  -> SS-06B
  -> SS-07B
  -> SS-08B
  -> SS-09B
```

The canonical lane/path-lease/barrier schedule is
[`WAVE_EXECUTION_PLAN.md`](../WAVE_EXECUTION_PLAN.md). This file defines worker
rules; the wave plan decides which rules may execute concurrently.

Worker rules specific to this cut:

1. Never add a V1 alias, adapter shim, fake default authority, or fallback to
   raw HIR to make a consumer compile.
2. Repair missing semantic facts at the producer. The active example is the
   missing explicit Group/empty-partition contract for an implicit global
   aggregate Project; the Algebra lowerer must keep rejecting it until Sema
   emits it.
3. Treat current owner GREEN separately from integration GREEN. Sema `87/87`
   and Algebra `4/4` do not qualify Pipeline, workspace, conformance, or CI.
4. Preserve unrelated dirty files. Do not run bulk formatting in the shared
   worktree; format/check only owned paths until the candidate lane.
5. Record base SHA, pre/post hashes, selected test count, exact command, and
   deferred consumer failures. A source scan or compile-only result is not a
   receipt.
6. Stop if the assigned change requires a reserved path or changes a frozen
   upstream contract. Amend the owner ticket before coding.

Minimum preflight:

```bash
git rev-parse HEAD
git status --short
git diff --check
rg -n "Algebra(Node|Op)V1|NormalizeOptionsV1|normalize_(query|relation)" crates
```

The legacy-reference scan is blast-radius evidence, not a command to repair all
hits in one patch. Follow the ticket order above.

## Current Agent Dispatch Cards

Use this table only for routing. The linked ticket's final current-state block
is the implementation authority; older sections are rationale and historical
receipts.

| Work package | Start gate | Primary writable owner paths | First proof to capture | Required handoff |
|---|---|---|---|---|
| [`SS-02G`](./02-surface-hir-semantic-shapes.md#next-ss--rr-checklist) | may start now; product source is read-only | new Receipt 02 public-API inventory proposal; HIR/Surface are frozen inputs | single four-argument relation-source constructor, name-span test enrollment, exact owned hashes | integrator writes Receipt 02 after frozen qualification |
| [`SS-03H`](./03-authority-types-modules-invalid-states.md#ss-03h-execution-routing) | may start now; current owners are GREEN | Types/Modules test leases; Sema work belongs to the combined Sema lane | forged topology/row witnesses reject exactly; capture and one-kernel proof are reviewed from the Sema lane | refreshed public-API/test inventories and Receipt 03 bound to one fingerprint |
| [`SS-03H-S + SS-04W`](./04-sema-kernel-and-document-transaction.md#ss-04w-final-current-state-supersession--2026-09-21) | may start now as one writer; no second Sema writer | Sema capture validator/oracle plus aggregate/budget producer paths listed in the wave plan | capture completeness, scalar `Group(empty)`, grouped partition identity, full immutable downstream/transport budget | aggregate-ready sealed document, exact/+1 budget proof, RR-03H/RR-04W inspection |
| [`SS-05C`](./05-relational-algebra-and-aggregate-semantics.md#rr-05c-final-current-state-audit--2026-09-21) | only after 04W contract and hashes freeze | Algebra source/tests and Receipt 05 | pure EXISTS currently leaves a residual EXISTS Filter; deep lowering is recursive/unbudgeted | complete operator/mutation matrix and exact sealed-budget lowering receipt |
| [`SS-06B`](./06-rule-graph-logic-contracts.md#ss-06b-final-current-state-supersession--2026-09-21) | only after 05C | Logic source/tests, Logic Cargo edge, architecture fixtures, Receipt 06 | `cargo check -p quanta-gqlang-v2-logic --all-targets` fails at deleted Algebra V1 imports | document-wide rule/graph contract, iterative budgeted SCC, no HIR/Algebra edge |
| [`SS-07B` from RR-07B](./07-semantic-planning-and-public-facade.md#rr-07b-authoritative-current-state-reconciliation--2026-09-21) | implementation only after 06B | Planning; integrator-owned Pipeline/facade/Cargo paths; Receipt 07 | Planning/Pipeline checks currently stop at Logic; V1 request/classifier and facade globs remain | semantic-only assembler/planner/facade, external consumer gate, removed transition edges |
| [`SS-08B` from RR-08B](./08-frontdoors-transport-and-resource-bounds.md#rr-08b-authoritative-current-state-reconciliation--2026-09-21) | only after Ticket 07 facade DTO freezes | CLI and diagnostics-stdio; parity tool; integrator-owned dependency/docs paths; Receipt 08 | current transport is unbounded brace framing and CLI ingress is unbounded | bounded NDJSON/CLI, typed diagnostics, parity, current legacy inventory zero |
| [`SS-09B` from RR-09B](./09-conformance-truth-sync-and-closeout.md#rr-09b-authoritative-current-state-reconciliation--2026-09-21) | implementation can prepare isolated schemas/validators; qualification waits for 01–08 candidate | Conformance, generated inventories/truth docs, CI/final-DAG integration, Receipts 09/closeout | V2 manifest/tests/scripts absent; Semgrep 13 findings; 95 allow attributes; no candidate SHA | exact candidate A proof, docs-only attestation B, hosted evidence and final-DAG receipt |

Ticket 00 has no worker task. Ticket 01 also has no standalone production
repair lane: each transition edge is deleted by its owning consumer ticket,
and Ticket 09 runs the final-mode architecture gate. The post-receipt Ticket 01
doc gate is rerun after Ticket 07 restores Pipeline compilation.

### Parallel-write boundary

The only safe immediate parallel split is:

1. Agent A owns SS-02G inventory preparation and does not edit product source.
2. Agent B owns Ticket 03 Types/Modules test-only mutation work.
3. Agent C is the sole Sema writer and owns both Ticket 03 capture/invocation
   proof and Ticket 04 aggregate/budget production.
4. Receipts 02-04, manifests, packet indexes, and the execution ledger are
   integrator-reserved while parallel agents run.

Do not run a Ticket 03 Sema writer beside Ticket 04: both require
`sema/src/document.rs` and `sema/tests/v3_document_transaction.rs`. Ticket 05
does not start until the combined Sema contract is frozen. Later tickets have
safe intra-wave splits documented in the wave plan; their consumer waves remain
ordered 05 -> 06 -> 07 -> 08 -> 09.

### Assignment manifest

Every delegated task must state all fields below before code changes:

```text
work_package=
base_sha=
preexisting_dirty_paths_and_hashes=
allowed_write_paths=
integrator_reserved_paths=
frozen_input_paths_and_hashes=
expected_red_command_and_result=
required_positive_and_negative_oracles=
required_deliberate_mutation=
acceptance_commands=
receipt_path=
deferred_consumer_failures=
```

If `allowed_write_paths` changes, stop and amend the ticket. A change to a
declared frozen input invalidates the lane; unrelated leased paths may advance
in parallel. Wave/workspace/candidate receipts still require a globally quiet
source fingerprint.

## Before Editing

1. Record exact `git rev-parse HEAD` and `git status --short`.
2. Read `../README.md`, `../FINAL_AUDIT.md`, `../TARGET_ARCHITECTURE.md`, ticket
   00, `../IMPLEMENTATION_REUSE_AUDIT.md`, and the assigned ticket completely.
3. Confirm every intended file is inside the ticket write set.
4. Stop if a reserved integration file or another active ticket's write set is
   required unexpectedly.
5. Reproduce the ticket's RED behavior before changing product code.
6. Confirm every existing path in the write set exists and every proposed new
   path has one canonical owner.
7. Mark the ticket `READY`, then `RED_CAPTURED`, in the execution ledger; never
   change the ticket's declared packet status from a worker branch.

## Implementation Rules

1. Repair the canonical producer. Do not patch a consumer to reinterpret raw
   HIR, synthesize default authority, or swallow an unresolved result.
2. Do not add a V1 compatibility wrapper unless ticket 00 explicitly lists it.
3. Do not introduce runtime/provider/SDK nouns into frontend contracts.
4. Do not weaken a diagnostic to make an old positive fixture pass.
5. Invalid states should be unrepresentable or rejected by a fallible owner
   constructor.
6. Tests must verify semantic payloads, not only enum variants or success exits.
7. Any newly declared CI/static rule requires a non-vacuity test.
8. Use the exact public type, test file, diagnostic code, and command names in
   the assigned ticket. A required rename is a packet amendment, not a worker
   choice.
9. Do not add ignored, quarantined, snapshot-only, text-grep-only, or
   discriminant-only tests as semantic acceptance.
10. Do not retain old and new representations simultaneously at handoff.
11. Before adding a module, algorithm, or test file, search the current owner
    crate. Classify the closest existing asset as `KEEP`, `MIGRATE`, `DELETE`,
    or `NEW` in the receipt and prove that the new work is not a second owner.
12. `MIGRATE` means preserve one algorithm while changing its input/output
    carrier. It does not mean copy the implementation and leave both active.

## Required Test Layers

1. owner unit test
2. owner negative test
3. cross-stage pipeline test
4. public front-door test when the seam is externally visible
5. one mutation/adversarial case proving the assertion is necessary

For a negative test, assert the stable diagnostic code and primary span; assert
secondary/edge spans where the ticket requires them. For a positive test,
assert the retained semantic payload, not merely `Ok`, process exit zero, or a
top-level variant.

## Execution Loop

1. Run the ticket's RED command on the frozen base and save the failure.
2. Implement only inside the declared write set.
3. Run the smallest owner test after each owner change.
4. Run every exact verification command in ticket order.
5. Inject each listed mutation one at a time, prove the named test fails, and
   revert the mutation.
6. Re-run the exact verification list after all mutations are reverted.
7. Generate the receipt and an owned-path diff summary.
8. Hand reserved-file patches and receipt to the integrator.

If a required executable is missing, record `BLOCKED_TOOLING` with the command
and error. Do not replace it with a weaker tool or mark the command skipped.

## Stop Conditions

Stop and escalate when:

1. the frozen semantic contract is insufficient
2. implementation would reverse the final dependency DAG
3. a product decision changes syntax or public result semantics
4. the same concept would gain a second owner
5. a test can only pass by restoring structural/open compile success
6. unrelated dirty changes overlap the assigned write set
7. the base SHA changes after RED capture
8. a named source/test path or public type must be changed to make the design
   compile
9. a required verification command selects zero tests or zero scan targets

## Handoff

Provide the ticket receipt schema from `README.md`, exact diagnostics for any
blocked RED/GREEN rail, and a file-scoped diff summary. Do not claim packet
closure.

The handoff must also state:

1. status reached in the ticket state machine
2. commits to integrate in order
3. reserved-file patch, if any
4. every mutation injected and killed
5. any known boundary not exercised locally
