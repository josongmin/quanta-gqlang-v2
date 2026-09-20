# Worker Start Here

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
