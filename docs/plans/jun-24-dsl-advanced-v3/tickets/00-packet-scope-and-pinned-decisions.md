# 00 Packet Scope And Pinned Decisions

## Goal

Freeze the architectural decisions for the `DSL Advanced v3` rewrite before any
implementation ticket starts.

## In Scope

1. syntax AST redesign
2. typed HIR redesign
3. binder/scope/type/algebra/planning redesign
4. CLI/LSP/formatter/proof-rail redesign inside this repo
5. hard deletion or quarantine of the current direct `v2` core path

## Out Of Scope

1. semantica runtime/provider/backend implementation
2. runtime substrate naming in the public language core
3. preserving source compatibility for the current default surface
4. partial feature parity stories that keep the old architecture alive

## Pinned Decisions

1. `D1`
   - `v3` is a new language core, not a `v2 SELECT++` extension.
2. `D2`
   - Generic HIR is the first shared semantic authority after syntax AST.
3. `D3`
   - Binder/scope resolution is mandatory before normalized algebra lowering.
4. `D4`
   - Type checking uses a closed operator/function registry with typed errors.
5. `D5`
   - Query normalization targets relational/logic algebra, not root-local IRs.
6. `D6`
   - Planning attaches to normalized query/program shape, not only root kind.
7. `D7`
   - Runtime/provider/backend nouns remain out of public frontend contracts.
8. `D8`
   - Compatibility is optional; architecture cleanliness is mandatory.
9. `D9`
   - `v2` direct syntax->typed->IR path is not preserved as a co-equal default.
10. `D10`
    - Unsupported advanced constructs reject explicitly; no silent downgrade to
      the `v2` subset.
11. `D11`
    - Legacy `quanta-gqlang-*` crates may be referenced for semantics, but not
      ported verbatim; `v3` re-derives ownership and boundaries in this repo.
12. `D12`
    - Ticket ownership is strict:
      `04` owns algebra nodes and normalization targets,
      `05` owns logic/source semantics and legality,
      `07` consumes normalized algebra/logic only for planning/truth/effects.
13. `D13`
    - Parser acceptance is structural only; binding, overload resolution, and
      type legality remain outside syntax and stay owned by `02`/`03`.
14. `D14`
    - The public frontend emits host-agnostic HIR/algebra/planning outputs; it
      does not mint semantica-native runtime DTOs or substrate-specific plans.
15. `D15`
    - Semantica integration happens through sibling exec-contract / host-spi /
      host-adapter seams, not by importing runtime substrate concerns into this
      repo.
16. `D16`
    - No lossy downgrade is allowed from `v3` algebra/logic semantics into the
      current fixed `RootSpecV1` request shell; unsupported serving shapes stay
      explicitly non-servable until the serving contract evolves.
17. `D17`
    - The current `ServingQueryRequestV1` / `RootSpecV1` shell is compat-only
      for existing `v2` serving shapes; it is not the semantic carrier for `v3`
      algebra/logic query structure.
18. `D18`
    - Repo-local frontend closure is not the same claim as end-to-end advanced
      serving readiness; the latter additionally requires sibling
      exec-contract/host-adapter/conformance proof in semantica.
19. `D19`
    - Reusable module/import/signature boundaries and packable query libraries
      are first-class architecture, not post-closeout tooling sugar.
20. `D20`
    - Result kinds, source locations, witness/path payloads, and explainability
      are semantic outputs of the language stack, not CLI-only presentation
      details.
21. `D21`
    - Cost visibility, performance rails, and incremental authoring workflows
      are mandatory DX requirements for `v3`, not optional afterthoughts.
22. `D22`
    - CLI, LSP, formatter, and proof/test tooling must share one compiler
      pipeline; shadow parser/sema implementations are explicitly forbidden.
23. `D23`
    - Correlated subqueries, nested result shapes, and path/proof
      materialization must have explicit frontend semantics and IR ownership;
      they are not post-processing sugar.
24. `D24`
    - Logical explain/provenance and semantic cost signals are frontend-owned,
      host-agnostic outputs; runtime physical plans and substrate profiling stay
      downstream.
25. `D25`
    - Nested/path forms use lexical, binder-visible scoping only; no hidden
      path-scoping or implicit capture algorithm may become semantic authority.
26. `D26`
    - Multiplicity, duplicate-elimination, and output-order semantics must be
      explicit and testable; accidental SQL bag leakage is not acceptable.
27. `D27`
    - Positive recursion uses explicit least-fixpoint semantics; recursive
      aggregates or unbounded witness/path materialization reject unless a
      legality rule explicitly admits them.
28. `D28`
    - Extension/model pack conflicts fail closed with deterministic diagnostics;
      no last-wins or import-order-sensitive override semantics are allowed.
29. `D29`
    - Shared result-shape structure is a frontend HIR concern owned by `01`;
      `03` types it, `04` normalizes relation producers, and `07` derives render
      semantics from it. No second shape IR may appear later.
30. `D30`
    - Module/import graph semantics are owned by `11`; binder `02` resolves
      names only against the frozen graph and may not invent import-order
      authority.
31. `D31`
    - Recursive witness/path materialization requires explicit boundedness,
      cycle-dedup identity, and tie-break rules; otherwise the shape rejects.
32. `D32`
    - Semantic cost/profile output must use a fixed, host-agnostic schema with
      stable field names for CLI/LSP/tests.
33. `D33`
    - Public compile surfaces must consume an explicit frozen frontend context
      for semantic authority (module graph, source schemas, pack state, and
      similar frontend-owned inputs). Empty-authority compile success is not a
      semantic-closure proof.
34. `D34`
    - Binder-visible locals must be type-checked under a lifted local type
      environment. Alias-dependent or param-dependent expressions may not
      silently bypass typing because the environment stayed open.
35. `D35`
    - Horn-rule source semantics require a dedicated lowering from shared HIR
      into recursive predicate/program structures. Parser/HIR presence alone is
      not rule support.
36. `D36`
    - Non-relational frontend units need an explicit compiled-artifact or
      explicit no-serve contract. They may not permanently bottom out in a
      relational-algebra placeholder reject.

## Current Blockers This Packet Solves

1. no generic expression AST/HIR
2. no binder model for aliases, params, or correlated scopes
3. no query algebra layer
4. no logic semantics / recursion discipline
5. no expression-level planning/truth/evidence model
6. no explicit semantic-authority contract for public compile surfaces
7. no typed local-env lifting from relation source schemas into alias/param
   scopes
8. no source horn-rule lowering into recursive predicate semantics
9. no explicit compiled-artifact boundary for standalone non-relational units

## Closure

1. all downstream tickets cite `D#`
2. no downstream ticket contradicts `D1` through `D32`

## Worker Packet

### Dependencies

1. none; this is the packet gate

### Mandatory Worker Outputs

1. every downstream ticket cites the owning `D#` decisions it depends on
2. packet-level ownership conflicts are removed before implementation starts
3. any newly discovered semantic ambiguity is pinned here before code work

### Proof Rails

1. packet docs stay internally consistent under link/read review
2. [tickets/README.md](./README.md) and
   [WORKER_START_HERE.md](./WORKER_START_HERE.md) reflect the same worker
   contract

### Reject Or Escalate

1. any ticket that tries to move runtime ownership into this repo
2. any ticket that creates duplicate authority across syntax, sema, IR, or
   planning
3. any closeout claim that skips updating this gate after semantics changed
