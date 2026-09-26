# ADR-0001: Consolidate retired frontend campaigns into current authority

- Status: Accepted
- Date: 2026-09-24
- Scope: `quanta-gqlang-v2` frontend repository

## Context

Five historical campaign trees accumulated 107 plan, ticket, worker-prompt,
ledger, and receipt files under `docs/plans/`:

1. Jun-9 repository startup recovery
2. Jun-10 `LET`/`WITH` composition
3. Jun-10 cross-repository follow-up coordination
4. Jun-24 DSL Advanced v3
5. Sep-21 SOTA++ frontend reconstruction

Those packets were useful while work was being scheduled. They later became a
second, inconsistent status system: source-bound test counts and open/closed
labels survived after the implementation and qualification source moved.
Historical execution prose is not an API, architecture, or proof authority.

## Decision

Delete `docs/plans/` and retain one durable decision record. Detailed campaign
history remains recoverable from Git through the parent of the deletion commit,
including qualified source `9612af51b7ea9b2438595447dd555d4c4c2659c4`.

Current authority is, in order:

1. repository source and Cargo workspace topology
2. [language and owner-stack reference](../language/README.md)
3. [frontend execution contract V3](../contracts/frontend-execution-v3.md)
4. [test hellgate ledger](../language/test_hellgate_ledger.md)
5. [local qualification procedure](../../tools/qualification/README.md)

Future temporary execution plans must not become long-lived implementation
authority. Promote durable architectural decisions into an ADR, public contract,
language reference, or executable owner test. Keep source-specific receipts
source-specific.

## Durable architecture decisions

### One frontend owner stack

The live stack has one owner for each stage: Surface, HIR, Binder, Types,
Modules, Sema, Algebra, Logic, Planning, Pipeline, and the curated frontend
contract. A feature must extend those owners instead of creating a shadow
parser, semantic checker, IR, planner, or registry.

### One public downstream boundary

`quanta-gqlang-v2-frontend-contract` is the public downstream facade. Direct
owner-crate dependencies are reserved for deliberate owner-local work, not for
creating another product-facing contract.

### Closed source compilation owns execution documents

`compile_source_document_v3` is the only closed source entry that can issue an
`ExecutionDocumentV3`. Resolved-only and open-analysis paths cannot mint source
or qualification digests. Public callers cannot assemble execution items or
supply their own digests.

### Deterministic, fail-closed semantics

Grammar, binding, typing, module resolution, normalized relational and logic
semantics, planning policy, diagnostics, and digest encoding are deterministic.
Unsupported or incomplete states reject with typed diagnostics or explicit
no-plan/no-payload dispositions; they do not guess, silently skip, or widen
into runtime authority.

### Composition remains explicit

Composition preserves source order, binding identity, producer/body families,
the exact `WITH` target, checked schemas, and normalized relations where
available. Representation by the frontend does not grant runtime support.

### Frontend and runtime ownership stay separate

This repository owns syntax, semantic analysis, normalized IR, planning
metadata, the source-bound V3 projection, CLI, and private diagnostics stdio.
It does not own runtime admission, provider/generation selection, leases,
budgets, physical execution, materialization, result receipts, deployment, or
activation. Those claims require downstream evidence bound to exact source
pins.

### Shipped front doors are bounded

The shipped process surfaces are the CLI and private diagnostics NDJSON stdio.
The diagnostics process is not an LSP or a runtime service. Both surfaces must
remain resource-bounded and behaviorally consistent with the shared Pipeline.

## Consolidated campaign outcomes

### Jun-9 repository startup recovery

The recovery restored the repo-local language and proof surfaces that had been
removed or narrowed, integrated them through the local pipeline, extracted
shared read-only analysis from front-door code, and localized test-only lint
exceptions. Its worker ordering and historical counts are retired.

### Jun-10 composition campaign

The original V1 `LET`/`WITH` working-set design and its old crate names were
superseded. The durable requirement is preserved by the current composition
contract: explicit ordered bindings and body identity, fail-closed name and
type handling, no hidden session state, and no implicit runtime
materialization.

### Jun-10 cross-repository follow-up

The qglang-local recovery and contract work was absorbed into the current owner
stack. Deletion waves, gateway cutover, SDK/backend wiring, and joint product
closeout belonged to Semantica and never became qglang runtime ownership.
Historical Semantica statuses in this repository are not current downstream
evidence.

### Jun-24 DSL Advanced v3

The campaign established the generic typed HIR, binder/type/module authority,
normalized algebra and logic, plan truth/evidence/effect metadata, hard legacy
surface cutover, module/pack semantics, context-first compilation, local type
closure, rule lowering, and explicit non-relational no-serve boundaries. Its
dated downstream handoff statements are retired; only current downstream
receipts can establish consumer state.

### Sep-21 frontend reconstruction and closeout

The reconstruction converged the owner stack, public facade, bounded front
doors, conformance inventory, adversarial owner tests, and qualification
tooling. Duplicate generated manifests and duplicate broad test matrices were
rejected when an executable owner already existed. Coverage gaps were closed
at the cheapest trustworthy owner boundary, with focused cross-owner tests only
where the behavior crossed owners.

The final pre-consolidation code-bearing source was
`9612af51b7ea9b2438595447dd555d4c4c2659c4`, tree
`4b614fae9aaf2c5eeca26f8f312095090e89dbd8`. Its clean committed-head
`just qualify-local` receipt recorded `just ci`, exit `0`, and 53 seconds.
Observed rails included 1,176/1,176 Nextest cases, 5/5 doctests, front-door
parity 8/8, external facade positive 1/1, privacy negatives 15/15, public API
89/89, final workspace architecture, both strict Clippy toolchains, deny, and
non-vacuous Semgrep. This is repository-local frontend proof only.

## Current proof and maintenance rules

1. Run the smallest affected owner test during iteration.
2. Run `just ci` for repository-wide local composition.
3. Run `just qualify-local` only from a clean committed HEAD when an exact
   source receipt is required.
4. Do not reuse an older SHA, test count, mutation result, downstream status,
   or dashboard as proof for changed source.
5. Keep historical injected-mutation claims distinct from normal adversarial
   fixtures and current regression tests.
6. Do not add sleeps, ignored tests, mutable global fixtures, production
   expected-value helpers, or weaker assertions to make a gate pass.
7. Downstream consumer, hosted CI, deployment, and activation evidence remain
   separate from repository-local qualification.

## Consequences

- There is no `docs/plans/` status surface to drift from the implementation.
- Detailed historical scheduling and receipts require Git history rather than
  a live documentation tree.
- Public and architectural claims have fewer authorities and clearer owners.
- A future campaign can create temporary coordination artifacts, but durable
  results must land in current documentation, ADRs, contracts, or executable
  tests before closeout.

## Reopen conditions

Reopen an architectural decision only when current source demonstrates one of
the following:

1. a reproducible semantic or ownership defect
2. a required public-contract change
3. a new runtime/frontend responsibility boundary
4. a missing executable oracle for a current public claim

Historical plan prose alone is not a reopen condition.
