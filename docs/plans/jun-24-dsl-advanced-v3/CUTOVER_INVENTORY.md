# v2 -> v3 Cutover Inventory (Ticket 09)

Status: legacy direct-path deleted, shared v3 frontend live.

## Current Head

`quanta-gqlang-v2` no longer ships the legacy direct `syntax -> sema -> ir`
crate stack. The crates `quanta-gqlang-v2-syntax`,
`quanta-gqlang-v2-sema`, and `quanta-gqlang-v2-ir` were physically deleted.
Their responsibility was not dropped. It was reseated into one v3 frontend:

1. surface grammar + formatter: `quanta-gqlang-v2-surface`
2. typed item family SSOT: `quanta-gqlang-v2-hir`
3. binding / schema / catalog authority: `binder`, `types`, `modules`
4. normalized frontend semantics: `algebra` + `logic`
5. host-agnostic plan / truth / effect contract: `planning`
6. one compiler service:
   semantic entrypoint `quanta-gqlang-v2-pipeline::compile_with_context`
   plus explicit structural/schema-only helpers
7. downstream consumer seam: `quanta-gqlang-v2-frontend-contract`

Cites: `D8` (architecture cleanliness mandatory), `D10` (fail-closed),
`D22` (one shared pipeline), `D24` (frontend/runtime boundary), `D32`
(fixed semantic-cost schema).

## Family Coverage

Current head v3 covers the old public language breadth through the shared
frontend, not through resurrected legacy crates.

Served by the shared pipeline today:

1. relational query
2. all FIND subkinds: `reachability`, `flow`, `slice`, `taint`, `path`,
   `points_to`, `summary`
3. `CHECK`
4. presentation verbs: `EXPLAIN`, `SHOW`, `DESCRIBE`, `SAMPLE`, `COMPARE`,
   `EXPORT`
5. catalog declarations: `VIEW`, `MODEL`, `PACK`, `PROFILE`, `GATE`,
   `RULEDEF`
6. meta directives: `USE PACK`, `REQUIRE CERTIFIED`
7. composition: `LET ... WITH ...`

Authority proof in-tree:

1. `crates/quanta-gqlang-v2-pipeline/tests/audit_18_families.rs`
2. `crates/quanta-gqlang-v2-surface/tests/v3_family_hellgate.rs`
3. `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs`

## Disposition

### 1. legacy parser/sema/ir crates

- `quanta-gqlang-v2-syntax`: deleted
- `quanta-gqlang-v2-sema`: deleted
- `quanta-gqlang-v2-ir`: deleted

These names are forbidden as downstream contract surface. They are not compat
APIs and must not be reintroduced to satisfy sibling workspaces.

### 2. frontend authority after the cutover

- parse / document / family formatting: `surface`
- typed top-level item + spans + ids: `hir`
- scope / symbol / module resolution: `binder` + `modules`
- schema-driven typing: `types`
- normalized relational semantics: `algebra` + `logic`
- plan / truth / effect / render / cost: `planning`
- full frontend compilation: `pipeline`

### 3. downstream ownership rule

- allowed: sibling repos consume the frontend through
  `quanta-gqlang-v2-frontend-contract`
- allowed: deliberate owner-local seam work may also depend on the underlying
  owner crates
- forbidden: semantica/sdk/runtime repos treating this repo as a workspace
  member or reviving deleted legacy crate edges
- forbidden: minting runtime DTOs or backend wiring in this repo

## Remaining Work

The main unresolved cutover is not language-family coverage inside this repo.
It is the downstream serving seam in `semantica-codegraph-v2`.

Still pending there:

1. delete `PublicIrRootV1` / `LoweringReadyIrV1` ingress assumptions
2. replace `RootSpecV1` as the default advanced-serving contract
3. port bridge and conformance rails to `CompiledProgramV1`
