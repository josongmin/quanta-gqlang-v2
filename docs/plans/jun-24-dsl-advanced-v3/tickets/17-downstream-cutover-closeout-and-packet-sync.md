# 17 Downstream Cutover Closeout and Packet Sync

Status: CLOSED on `2026-06-26` — synced to the landed semantica D5 receipt.

## Closeout Receipt (`2026-06-26`)

Both gating inputs are now satisfied, so the terminal sync is performed:

1. qglang ticket `16` is closed; its repo-local proof receipt is refreshed on the
   exact head being synced (build/test/clippy/fmt green, artifact-hygiene rail
   green, full mutation hardening landed).
2. semantica D5 serving cutover landed on its `main`: tickets `01`–`06` plus the
   adversarial-audit structural completion (receipt `…77d2f34983`, D5 head
   `…f189330b16`). Legacy serving authority removed; `exec-conformance` orphan
   crate deleted; `LoweredQueryPlanV1` collapsed; `ServingQueryRequestV1` /
   `RootSpecV1` + the dead spec tree deleted (true zero dual-truth); the bridge
   lowers relational serving into `ServingProgramRequestV1` with an EXHAUSTIVE
   `served_family_v1()` label.

Honest residual (recorded separately, NOT used to claim false green): D5 ticket
`07` lists only ambient / doc-hygiene / other-lane items. The one execution-side
residual is that the `group_gqlang_public_path` preflight assertion runs under a
test closure (`runtime-executor-surface` → `quanta-taint-solver` /
`quanta-adapters-analysis-facade`) currently churning under a separate concurrent
owner — an AMBIENT test-closure state, not a gqlang/serving-correctness blocker.
End-to-end advanced-serving green therefore still waits on that ambient closure,
but the serving cutover correctness is landed.

## Goal

Re-synchronize the qglang umbrella packet with the final semantica D5 cutover
receipt so the packet becomes honest SSOT again after downstream execution
finishes.

## Current-Head Truth

1. this repo already documents the new frontend-only ownership boundary
2. ticket `10` now serves as the downstream umbrella, but it is not the
   execution packet
3. the semantica execution packet lives in
   `packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/`
4. qglang repo-local proof rails were refreshed on `2026-06-25` and are green
5. packet sync must stay blocked until:
   - semantica D5 proof rails are green
   - qglang ticket `16` receipt is refreshed against the exact qglang head
     being synchronized

## Deliverables

1. update the packet SSOT files after semantica D5 closes:
   - `README.md`
   - `tickets/README.md`
   - `WORKER_START_HERE.md`
   - `SERVING_HANDOFF.md`
   - `10-semantica-serving-contract-handoff.md`
   - `16-current-main-proof-and-artifact-hygiene.md`
2. restate current-head truth using the final downstream receipt, not earlier
   intermediate cutover language
3. explicitly gate any “end-to-end advanced-serving closure” claim on the final
   semantica D5 proof receipt

## Inputs Required Before Closeout

1. ticket `16` refreshed with the repo-local proof receipt preserved on the
   exact qglang head being synced
2. semantica D5 packet closed through `06-proof-closeout-mainline-foldback-and-worktree-cleanup`
3. ambient blockers, if any, recorded separately from ticket-owned failures

## Definition Of Done

1. qglang packet files and semantica D5 packet describe the same current-head
   truth
2. no packet file implies downstream closure without a green D5 receipt
3. ticket `10` remains the umbrella, while `17` becomes the terminal sync lane
4. stale dirty-tree language from older `16` snapshots is gone
