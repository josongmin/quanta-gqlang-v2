# 10 Semantica Serving Contract Handoff

Status: semantica D5 execution packet open; qglang ticket `10` remains the
downstream umbrella and packet SSOT

## Goal

Complete the breaking downstream cutover so semantica finishes deleting the
legacy public request/root facade and serves downstream only through the `v3`
frontend bundle seam.

## Current-Head Truth

1. this repo no longer ships `quanta-gqlang-v2-syntax`,
   `quanta-gqlang-v2-sema`, or `quanta-gqlang-v2-ir`
2. semantica bridge/host-adapter paths already depend on
   `quanta-gqlang-v2-frontend-contract`
3. semantica `query_bridge.rs` already compiles text through the frontend
   contract structural bundle seam and lowers relational serving into
   `ServingProgramRequestV1`
4. legacy request/root helpers in semantica remain as fail-closed compat
   residue instead of silently fabricating stale request/root authority
5. semantica public/query-owner facades already expose the
   `ServingProgramRequestV1` rail through
   `lower_query_text_to_program_request_v1` and `execute_query_text_v1`
6. the semantica D5 packet is still open and explicitly tracks remaining work
   across runtime public ingress, runtime executor, ingress-bridge/host-adapter
   compat collapse, exec-conformance, and workspace/member delete-prep closure
7. therefore the real task is not raw bridge bootstrap; it is downstream
   execution-packet closure plus final qglang packet re-sync after semantica
   produces a green D5 receipt

Authoritative detail lives in:

1. [../SERVING_HANDOFF.md](../SERVING_HANDOFF.md)
2. [../CUTOVER_INVENTORY.md](../CUTOVER_INVENTORY.md)

## Execution Packet

1. this repo keeps ticket `10` as the downstream umbrella and packet SSOT
2. semantica executes the owner-local cutover through the D5 packet:
   [D5_GQLANG_V3_CUTOVER_PACKET/README.md](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/README.md)
3. qglang-side closeout companions are:
   - [16-current-main-proof-and-artifact-hygiene](./16-current-main-proof-and-artifact-hygiene.md)
   - [17-downstream-cutover-closeout-and-packet-sync](./17-downstream-cutover-closeout-and-packet-sync.md)

## Deliverables

1. explicit rule that semantica may reference this repo only as a downstream
   consumer of the `v3` frontend seam
2. explicit delete/rewrite list for the remaining runtime ingress and compat
   exports that still expose `ServingQueryRequestV1` or deleted
   `PublicIrRootV1`-named helpers
3. exact breaking sequence from flat public facade to owner-local compat
   residue to eventual deletion
4. exact test migration plan for public-facade/source-contract rails

## Child Ticket Dependency Table

| lane | file | depends on | role |
| --- | --- | --- | --- |
| D5-00 | [00-scope-and-pinned-decisions](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/00-scope-and-pinned-decisions.md) | none | freezes integration rules and forbidden authority |
| D5-01 | [01-runtime-public-ingress-authority-cutover](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/01-runtime-public-ingress-authority-cutover.md) | D5-00 | removes legacy public ingress authority |
| D5-02 | [02-runtime-executor-program-request-cutover](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/02-runtime-executor-program-request-cutover.md) | D5-01 | rewires runtime executor to program-request authority |
| D5-03 | [03-ingress-bridge-and-host-adapter-compat-collapse](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/03-ingress-bridge-and-host-adapter-compat-collapse.md) | D5-02 | collapses compat root/request helpers and host-adapter residue |
| D5-04 | [04-exec-conformance-owner-surface-and-guard-rail-rewrite](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/04-exec-conformance-owner-surface-and-guard-rail-rewrite.md) | D5-03 | rewrites conformance, owner-surface docs, and runtime guards |
| D5-05 | [05-workspace-membership-manifest-and-delete-prep-closure](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/05-workspace-membership-manifest-and-delete-prep-closure.md) | D5-04 | closes workspace/member and manifest delete-prep blockers |
| D5-06 | [06-proof-closeout-mainline-foldback-and-worktree-cleanup](../../../../../semantica-codegraph-v2/packages/analysis/quanta-v2/D5_GQLANG_V3_CUTOVER_PACKET/06-proof-closeout-mainline-foldback-and-worktree-cleanup.md) | D5-05 | final proof, foldback, and worktree cleanup |
| 16 | [16-current-main-proof-and-artifact-hygiene](./16-current-main-proof-and-artifact-hygiene.md) | none | classifies qglang dirty-tree truth and proof hygiene |
| 17 | [17-downstream-cutover-closeout-and-packet-sync](./17-downstream-cutover-closeout-and-packet-sync.md) | D5-06, 16 | syncs qglang packet SSOT to final semantica receipt |

## Pinned Integration Rules

1. semantica may reference this repo
2. semantica may not absorb this repo as a workspace member
3. semantica may not depend on deleted `syntax` / `sema` / `ir` crates
4. advanced `v3` shapes may not be flattened back into `RootSpecV1::Select`
5. runtime DTOs remain semantica-owned, not frontend-owned
6. the preferred downstream dependency is
   `quanta-gqlang-v2-frontend-contract`, not a raw scatter of owner crates

## Breaking Plan

1. keep `quanta-gqlang-v2-frontend-contract` as the single frontend ingress
   dependency on the active bridge path
2. preserve the flat public-facade cutover already landed in semantica, then
   remove the remaining public/default legacy request helpers from runtime
   ingress and compat residue:
   `ServingQueryRequestV1`,
   `execute_serving_query_request_v1`,
   `gqlang_public_serving_request_from_text_surface_v1`,
   `lower_query_text_to_request_v1`,
   `lower_query_text_to_public_ir_root_v1`, and
   `lower_public_ir_root_to_request_v1`
3. keep any remaining request/root compat helpers owner-local only until the
   runtime compat rail is physically deleted
4. pin callers to `lower_query_text_to_program_request_v1`,
   `gqlang_public_program_request_from_text_surface_v1`,
   `execute_serving_program_request_v1`, or `execute_query_text_v1`
5. rewrite public-facade/source-contract rails around the `v3` bundle seam
6. delete the residual request-native ingress once no public caller still
   depends on it

## Definition Of Done

1. semantica references this repo only through the allowed `v3` surfaces
2. no flat semantica public facade still exports deleted root helpers or legacy
   request-native serving as default authority
3. the detailed current-head cutover in [../SERVING_HANDOFF.md](../SERVING_HANDOFF.md)
   is still true
