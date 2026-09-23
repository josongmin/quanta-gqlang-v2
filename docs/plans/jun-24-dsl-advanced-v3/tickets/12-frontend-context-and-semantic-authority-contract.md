# 12 Frontend Context And Semantic Authority Contract

> Historical 2026-06-24 snapshot. “Current Main” below refers to that dated
> source, not today's repository. Use the [current language authority](../../../language/README.md).

Status: `Materially Implemented On Current Main`

## Current Main Receipt

As of `2026-06-24`, the correctness seam owned by this ticket is live:

1. `FrontendContextV1` carries frozen `ModuleGraphV1` plus schema/type
   authority
2. `compile_with_context`, `compile_item_with_context`, and
   `compile_document_with_context` are the explicit semantic entrypoints
3. `compile_*_structural` and `compile_*_with_schema` remain explicit
   non-semantic helpers and must not be reported as semantic proof
4. `frontend-contract` re-exports the context-first compile surface
5. proof rails:
   `v3_frontend_context.rs`,
   `frontend-contract/tests/downstream_contract.rs`
6. verified with:
   `cargo test -p quanta-gqlang-v2-pipeline`,
   `cargo test -p quanta-gqlang-v2-frontend-contract`,
   `cargo test --workspace`,
   `cargo clippy --workspace --all-targets`

Honest residual:

1. there is no dedicated context fingerprint/caching API yet; current closure
   is semantic-authority correctness, not IDE cache-key design

## Goal

Replace empty-authority compile defaults with an explicit, frozen frontend
context that carries the semantic authority a `v3` compile actually depends on.

## Why

The pre-closeout gap was public compile paths defaulting to an empty module
graph and empty type environment. That is acceptable for structural parse/lower
tests, but it is not acceptable as the semantic default for a SOTA-grade query
frontend.

Without an explicit frontend context:

1. green compile receipts can mean "semantic authority missing" rather than
   "query is valid"
2. binder, types, modules, and pipeline can each invent partial fallback
   assumptions
3. downstream consumers cannot tell whether a compile was context-complete or
   context-thin
4. incremental authoring/caching has no canonical semantic input bundle

## Deliverables

1. `FrontendContextV1` or equivalent immutable semantic-authority bundle
2. explicit sub-authorities inside that bundle:
   - frozen module/import graph handle
   - relation/source schema catalog
   - pack/manifest view where relevant
   - feature/profile switches only if frontend-owned
3. public compile entrypoints that take the context explicitly
4. explicit split between:
   - parse/shape-only entrypoints
   - semantically-authoritative compile entrypoints
5. deterministic context hashing / identity story suitable for caching and IDE
   reuse
6. typed diagnostics or typed API boundaries for missing required authority
7. downstream facade updates in `frontend-contract`
8. proof rails showing context-complete and context-missing behavior separately

## Pinned Semantics

1. semantic compile authority is explicit input, not ambient global state
2. a compile with missing semantic authority is not proof of semantic validity
3. frontend context is immutable during one compile
4. module/import/type authority is frontend-owned only up to host-agnostic
   language semantics; runtime substrate authority stays out
5. parse/format/structural HIR APIs may remain context-free, but semantic
   compile APIs may not pretend that context-free success is semantic closure
6. downstream consumers must be able to inspect whether a compile ran with full
   authority or a deliberately limited mode

## Non-Goals

1. this ticket does not define the full module graph semantics:
   ticket `11`
2. this ticket does not define the closed type universe:
   ticket `03`
3. this ticket does not by itself solve alias/local type closure:
   ticket `13`
4. this ticket does not define semantica runtime request DTOs

## Scenarios Unlocked

1. the same query compiled under two different schema catalogs produces
   deterministic, inspectable semantic differences
2. CLI/LSP can cache results keyed by explicit semantic authority rather than
   source text alone
3. a downstream consumer can intentionally call a structural-only path without
   conflating that with a semantically-checked compile
4. proof rails can assert "missing authority rejected" instead of accepting
   false-green compiles

## Definition Of Done

1. public semantic compile surfaces no longer treat empty authority as the
   default proof mode
2. the semantic input bundle is explicit, immutable, and inspectable
3. `frontend-contract` exposes the new context-first compile surface
4. green compile receipts can now distinguish structural success from semantic
   success

## Dependencies

1. requires `00`
2. depends on `01`, `02`, and `03`
3. closes with `13`, `14`, `15`, and `08`
4. consumes module/pack authorities defined later by `11` without redefining
   them

## Likely Touch Points

1. `crates/quanta-gqlang-v2-pipeline/src/compile.rs`
2. `crates/quanta-gqlang-v2-types/src/env.rs`
3. `crates/quanta-gqlang-v2-binder/src/module_graph.rs`
4. `crates/quanta-gqlang-v2-frontend-contract/src/lib.rs`
5. CLI/LSP entrypoints that currently call context-free compile defaults

## Mandatory Worker Outputs

1. one explicit frontend semantic-authority bundle
2. one context-first public compile API
3. one explicit structural-only versus semantic compile distinction

## Proof Rails

1. add a rail that proves semantic compile rejects or classifies missing
   authority explicitly
2. add a rail that proves downstream facade exports the context-first seam
3. add a rail that identical source plus different contexts yields different,
   deterministic semantic outputs where expected

## Reject Or Escalate

1. hidden global schema/module state
2. treating empty authority compile success as semantic proof
3. runtime substrate/provider/SDK nouns leaking into the frontend context
