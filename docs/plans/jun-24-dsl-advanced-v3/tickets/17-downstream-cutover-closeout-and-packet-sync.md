# 17 Downstream Cutover Closeout and Packet Sync

Status: open; blocked on semantica D5 green receipt plus exact-head packet
re-sync

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
