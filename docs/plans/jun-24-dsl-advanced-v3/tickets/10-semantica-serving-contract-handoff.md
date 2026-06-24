# 10 Semantica Serving Contract Handoff

## Goal

Pin the breaking downstream cutover that removes semantica's stale dependency on
the deleted `v2` direct path and reseats downstream serving on the `v3`
frontend bundle seam.

## Current-Head Truth

1. this repo no longer ships `quanta-gqlang-v2-syntax`,
   `quanta-gqlang-v2-sema`, or `quanta-gqlang-v2-ir`
2. semantica still has manifests and bridge code that assume those crates exist
3. therefore the old handoff story is obsolete; the real task is a **breaking
   downstream cutover**, not a compat extension

Authoritative detail lives in:

1. [../SERVING_HANDOFF.md](../SERVING_HANDOFF.md)
2. [../CUTOVER_INVENTORY.md](../CUTOVER_INVENTORY.md)

## Deliverables

1. explicit rule that semantica may reference this repo only as a downstream
   consumer of the `v3` frontend seam
2. explicit delete list for stale sibling references to deleted `v2` crates
3. exact breaking sequence from `compile_program` to sibling contract to
   host-adapter lowering
4. exact test migration plan for bridge and conformance rails

## Pinned Integration Rules

1. semantica may reference this repo
2. semantica may not absorb this repo as a workspace member
3. semantica may not depend on deleted `syntax` / `sema` / `ir` crates
4. advanced `v3` shapes may not be flattened back into `RootSpecV1::Select`
5. runtime DTOs remain semantica-owned, not frontend-owned
6. the preferred downstream dependency is
   `quanta-gqlang-v2-frontend-contract`, not a raw scatter of owner crates

## Breaking Plan

1. cut deleted `v2` crate refs from semantica manifests
2. make `quanta-gqlang-v2-frontend-contract` the single frontend ingress
   dependency
3. introduce a new sibling `v3` serving contract instead of widening
   `RootSpecV1`
4. port bridge code from `PublicIrRootV1` lowering to `CompiledProgramV1`
   lowering
5. port host-adapter/runtime ingress to the new contract
6. rewrite bridge/conformance rails around the `v3` bundle seam

## Definition Of Done

1. semantica references this repo only through the allowed `v3` surfaces
2. no stale deleted-crate reference remains
3. the detailed current-head cutover in [../SERVING_HANDOFF.md](../SERVING_HANDOFF.md)
   is still true
