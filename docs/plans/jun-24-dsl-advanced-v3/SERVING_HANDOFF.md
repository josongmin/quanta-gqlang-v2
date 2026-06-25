# Semantica Serving Contract Handoff (Ticket 10)

Status: downstream cutover LANDED on semantica `main` (D5 receipt `…77d2f34983`,
head `…f189330b16`); serving-correctness closed, only an ambient test-closure
residual remains (see ticket `17`). `ServingQueryRequestV1` / `RootSpecV1` and the
dead spec tree are deleted; relational serving lowers into `ServingProgramRequestV1`
with an exhaustive `served_family_v1()` label; the `exec-conformance` orphan is removed.

## Purpose

Define the post-`09` downstream cutover for `semantica-codegraph-v2`.

Question this document answers:

1. should `semantica` reference `quanta-gqlang-v2` at all?
2. if yes, what exact surface may it reference?
3. what must be deleted now that the `v2` direct path is gone?

Short answer:

1. **yes, `semantica` may reference this repo**
2. **no, it must not absorb this repo as a workspace member or runtime owner**
3. **all deleted `syntax` / `sema` / `ir` references must be cut and replaced by
   the `v3` pipeline bundle seam**

## Current-Head Facts

Local current head (`quanta-gqlang-v2`):

1. the sole frontend is `quanta-gqlang-v2-pipeline`
2. `compile_with_context` is the semantic frontend entrypoint
3. explicit non-semantic helpers remain available:
   `compile_program_structural`,
   `compile_item_structural`,
   `compile_document_structural`,
   `compile_program_with_schema`,
   `compile_item_with_schema`,
   `compile_document_with_schema`
4. the legacy direct crates `quanta-gqlang-v2-syntax`,
   `quanta-gqlang-v2-sema`, and `quanta-gqlang-v2-ir` are physically deleted
5. `CompiledProgramV1` is the load-bearing downstream bundle:
   - `item()` -> parsed surface item
   - `compiled()` -> family-specific compiled result
   - `algebra()` -> normalized algebra root for relational queries
   - `plan()` -> host-agnostic public plan when the item has one
   - standalone decl/rule units may compile as explicit no-plan artifacts

Semantica current head has landed the canonical dependency seam and
program-request rail, but not the full downstream closeout:

1. semantica still keeps `quanta-gqlang-v2-ingress-bridge` and
   `quanta-gqlang-v2-exec-conformance` as live workspace members; workspace
   member cleanup is not current-head truth
2. the active bridge and host-adapter paths depend on the canonical sibling
   seam `quanta-gqlang-v2-frontend-contract`
3. `query_bridge.rs` no longer parses through the deleted
   `syntax -> sema -> ir -> PublicIrRootV1` stack; canonical text ingress now
   compiles via the frontend contract structural bundle seam and lowers to
   `ServingProgramRequestV1`
4. legacy `lower_query_text_to_request_v1`,
   `lower_query_text_to_plan_v1`,
   `lower_public_ir_root_to_request_v1`, and
   `lower_public_ir_root_to_plan_v1` remain only as fail-closed compat residue
5. `quanta-runtime-public-ingress-surface` already owns
   `execute_serving_program_request_v1`; the request-native
   `execute_serving_query_request_v1` rail survives as residual compat
6. `quanta-sdk-query-backend-owner` already exposes
   `lower_query_text_to_program_request_v1`, `execute_query_text_v1`, and
   `ServingProgramRequestV1` on the program-request rail
7. semantica D5 still tracks open work across runtime public ingress, runtime
   executor, ingress-bridge/host-adapter compat collapse, exec-conformance,
   and workspace/member delete-prep closure

This means the canonical dependency seam and the core program-request ingress
rail exist, but the downstream packet is not closed. Remaining work is broader
than flat public-facade cleanup and stays owned by the semantica D5 execution
packet.

## Boundary Decision

Pinned rule:

1. `semantica` **does** consume this repo, but only as a downstream frontend
   bundle consumer
2. `quanta-gqlang-v2` remains frontend-only owner
3. runtime substrate, provider routing, SDK backend wiring, and runtime-native
   DTOs stay semantica-owned
4. this repo must not be pulled in as a semantica workspace member
5. deleted `v2` crates are not a compat contract and must not be resurrected to
   satisfy sibling manifests

## Allowed Downstream Reference Surface

Preferred reference surface:

1. **Primary sibling seam: `quanta-gqlang-v2-frontend-contract`**
   - re-exports `compile_with_context`
   - re-exports the explicit structural/schema-only helpers
   - re-exports `FrontendContextV1`
   - re-exports `CompiledProgramV1` / `CompiledItemV1`
   - re-exports `NoServeReasonV1`
   - re-exports the host-agnostic HIR / algebra / planning / type-env bundle
   - exposes `compiled_program_item_v1(...)` for family inspection

Underlying owner crates remain frontend-owned, but sibling repos should not
take direct dependencies on them unless they are doing deliberate owner-local
work on the seam itself.

## Forbidden Downstream Reference Surface

1. deleted `quanta-gqlang-v2-syntax`
2. deleted `quanta-gqlang-v2-sema`
3. deleted `quanta-gqlang-v2-ir`
4. direct dependence on root-local `PublicIrRootV1` / `LoweringReadyIrV1`
   compat shapes as if they were still current-head authority
5. any plan to make `quanta-gqlang-v2` a semantica workspace member
6. any plan to mint semantica-native runtime DTOs in this repo
7. any `RootSpecV1::Select++` mutation that tries to smuggle `v3` algebra into
   the old flat compat shell

## Breaking Cutover Plan

### Phase 1: sever stale dependency edges

Status: partially landed; workspace-member cleanup is still open

Semantica changes:

1. remove `quanta-gqlang-v2-ingress-bridge` and
   `quanta-gqlang-v2-exec-conformance` from
   `packages/analysis/quanta-v2/Cargo.toml` workspace `members`
2. replace scattered direct owner-crate path deps with
   `quanta-gqlang-v2-frontend-contract` as the primary frontend dependency
3. add direct owner-crate deps only where the sibling seam is still missing a
   required read-only helper

Intentional effect:

1. semantica compile will break immediately at any code path still expecting
   `PublicIrRootV1`
2. that break is desired because it flushes all hidden `v2` assumptions into
   the type checker at once

### Phase 2: replace the ingress bridge front door

Status: partially landed; canonical ingress exists but legacy shims still
survive

Delete the old ingress story:

1. `lower_query_text_to_public_ir_root_v1`
2. `lower_public_ir_root_to_request_v1`
3. `lower_public_ir_root_to_plan_v1`
4. any helper whose authority starts from `PublicIrRootV1`

Replace it with a `v3`-native ingress story:

1. parse + bind + type + normalize through
   `quanta_gqlang_v2_pipeline::compile_with_context`
2. new semantica-owned front door accepts `CompiledProgramV1`, not
   `PublicIrRootV1`
3. family classification keys off `CompiledProgramV1::compiled()` and
   `ProgramItemHirV1`, not the deleted root enum
4. relational lowering keys off `CompiledProgramV1::algebra()` plus
   `CompiledProgramV1::plan()`
5. stage rejects propagate as `PipelineDiagnosticV1`-backed fail-closed ingress
   errors

### Phase 3: evolve the sibling serving contract

Status: partially landed for the relational serving family; residual request
compat still exists for stale callers

Breaking change is explicit:

1. `RootSpecV1` is no longer the primary language-serving contract
2. semantica introduces a new sibling-owned `v3` request contract in the
   exec-contract family
3. that contract carries the frontend-owned semantics without runtime leakage:
   - compiled item family
   - normalized algebra when present
   - host-agnostic `PublicPlanV1`
   - source-scope / working-set / compare / export metadata when present
4. `RootSpecV1` either:
   - moves under an explicit `compat` namespace, or
   - is deleted entirely if no legacy traffic remains

Non-negotiable rule:

1. do not widen `RootSpecV1::Select`
2. do not encode `v3` algebra as flattened pseudo-filters / pseudo-columns
3. do not call that compat

### Phase 4: host adapter owns runtime lowering

Status: partially landed for `ServingProgramRequestV1`; request-native compat
execution and non-relational family closure still remain open

Semantica host-adapter changes:

1. lower the new sibling `v3` contract into runtime-native plans
2. inspect `PublicPlanV1` for truth / effect / result-kind / witness policy
3. inspect `AlgebraNodeV1` only on the semantica side of the seam
4. reject unsupported families or shapes fail-closed

Reject source:

1. `PipelineDiagnosticV1` for parse/bind/type/modules/algebra/logic rejects
2. `PublicPlanV1.cost.unsupported_reason_code` for typed non-servable mixes
3. semantica-owned contract rejection codes when runtime support is absent

### Phase 5: replace the tests, not just the code

Status: partially open

Semantica tests that must be rewritten around the `v3` seam:

1. `quanta-gqlang-v2-ingress-bridge/tests/query_plane_bridge.rs`
2. `quanta-gqlang-v2-ingress-bridge/tests/composition_query_bridge.rs`
3. `quanta-gqlang-v2-exec-conformance/tests/query_ingress_full_loop.rs`
4. `quanta-gqlang-v2-exec-conformance/tests/declaration_meta_ingress_full_loop.rs`
5. `quanta-gqlang-v2-exec-conformance/tests/production_*`
6. any helper that manufactures `PublicIrRootV1` directly

New test authority:

1. text input + explicit `FrontendContextV1` ->
   `compile_with_context` -> `CompiledProgramV1`
2. bundle -> sibling `v3` request contract
3. contract -> host adapter lowering
4. unsupported shapes -> typed fail-closed rejection

Remaining semantica cleanup after the core cutover:

1. stop exporting `ServingQueryRequestV1`,
   `execute_serving_query_request_v1`,
   `gqlang_public_serving_request_from_text_surface_v1`, and deleted
   `PublicIrRootV1`-named lowerers from the flat public SDK/query facades
2. keep any remaining legacy request/root helpers owner-local only, not default
   public surface
3. rewrite source-contract tests so they pin `text -> ServingProgramRequestV1`
   and `text -> execute_query_text_v1`, not `text -> ServingQueryRequestV1`
4. delete the residual request-native runtime ingress once no public caller
   still depends on it

## Family Mapping Rules

The new bridge must classify by `CompiledItemV1`, not by deleted root carriers.

1. `RelationalQuery`
   - servable only through the normalized algebra + plan bundle
2. `GraphQuery`
   - servable only if the sibling `v3` contract and runtime own that family
   - otherwise reject explicitly
3. `Legality`
   - servable only if the sibling contract has an alert/proof lane for it
4. `Presentation`
   - `EXPLAIN` / `SAMPLE` / `COMPARE` / `EXPORT` are not automatically servable
   - each needs explicit sibling/runtime ownership
5. `CatalogRegistration`
   - no plan; ingress must not pretend it is a runtime query
6. `Composition`
   - the body may carry a plan, but the bridge must use the compiled body
     semantics directly
   - no deleted `CompositionIrV1` dependency may return
7. `Meta`
   - `USE PACK` and `REQUIRE CERTIFIED` stay explicit effect/gate items
   - they are not query roots
8. `ValueArtifact`
   - no plan; servable only if a future sibling contract explicitly owns
     declaration artifacts
   - otherwise preserve the explicit no-serve outcome
9. `RuleArtifact`
   - no runtime plan at this seam
   - servable only if a future sibling contract explicitly owns rule-artifact
     execution
   - otherwise preserve the explicit no-serve outcome

## Migration Order

1. cut semantica manifest refs to deleted crates
2. cut semantica nested workspace admission for the external frontend consumer
   crates
3. add pipeline-based compile entrypoint
4. introduce new sibling `v3` contract
5. port ingress bridge to `CompiledProgramV1`
6. port host adapter to the new contract
7. rewrite bridge and conformance rails
8. only then decide whether the explicit structural/schema-only helpers should
   stay public or narrow further

## Definition Of Done

1. semantica references this repo only through the allowed `v3` surfaces
2. no semantica manifest references deleted `syntax` / `sema` / `ir` crates
3. no semantica ingress code starts from `PublicIrRootV1`
4. `RootSpecV1` is no longer pretending to be the default advanced-serving seam
5. all advanced-serving claims route through the sibling `v3` contract
6. unsupported shapes reject explicitly and deterministically
7. this repo remains frontend-only and does not absorb runtime concerns
