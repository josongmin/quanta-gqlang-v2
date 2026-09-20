# Sep 21 — QGLang SOTA++ Frontend Reconstruction

Status: `EXECUTION ACTIVE — TICKETS 01-02 VERIFIED; TICKET 03 ACTIVE; PACKET NOT COMPLETE`

Audit base: `main@e896a11024bd5a1e1ef03c094a70032f4e6c765f`

Audit date: `2026-09-21`

Execution started after the audit. The original audit base remains authoritative
for before/after comparisons. External integration commits `96eab779` and
`fcf106b` are recorded in `EXECUTION_LEDGER.md`; neither is a clean Ticket 03
verification boundary. Tickets 04-09 remain unverified, and only ticket 09 may
declare packet closure.

This packet supersedes current-head completion claims in the Jun-24 packet. It
does not erase historical receipts. It records the delta between those receipts
and the behavior of the audited source.

## Verdict

The repository is not abandoned and is not a toy frontend. Lexer, parser, HIR,
binder, type registry, algebra, recursive-logic algorithms, planning vocabulary,
CLI, and diagnostics transport all contain substantial implementations.

It is not semantically closed. The exact status is:

> Components are substantially implemented, but the public pipeline lacks one
> document-scoped bound/typed semantic authority. Raw HIR is reinterpreted by
> multiple downstream consumers, producing semantic miscompilation and false
> completion claims.

The repair is not 37 local patches. It is one ownership reconstruction:

1. syntax HIR describes syntax only
2. modules resolves document/module/catalog authority
3. a new semantic owner binds and types the whole document
4. algebra and logic consume sealed semantic contracts
5. planning consumes verified contracts, never raw HIR
6. one curated facade feeds every front door

## Packet Files

1. [FINAL_AUDIT.md](./FINAL_AUDIT.md) — deduplicated findings and fresh local receipt
2. [IMPLEMENTATION_REUSE_AUDIT.md](./IMPLEMENTATION_REUSE_AUDIT.md) — current-head keep/migrate/delete/build census and duplicate-work guard
3. [TARGET_ARCHITECTURE.md](./TARGET_ARCHITECTURE.md) — ownership, dependency direction, public contracts
4. [tickets/README.md](./tickets/README.md) — execution order and ticket index
5. [tickets/WORKER_START_HERE.md](./tickets/WORKER_START_HERE.md) — worker and integrator protocol

## Root Cause Summary

| RCA | Canonical cause | Primary owner |
|---|---|---|
| R1 | string-tagged relation HIR has no canonical stage semantics | HIR + surface |
| R2 | no sealed bound/typed semantic IR exists | new sema owner |
| R3 | compilation is item-at-a-time, not a document transaction | sema + pipeline |
| R4 | planning reclassifies raw HIR instead of consuming proof contracts | planning |
| R5 | invalid authority/type/algebra states are publicly constructible | modules + types + algebra |
| R6 | front doors and proof gates do not share one current authority | facade + CLI/service + CI |

## Pinned Decisions

1. Introduce `quanta-gqlang-v2-sema` as a new implementation over the current
   HIR/binder/types/modules crates. Do not restore or copy a deleted legacy crate.
2. Make one breaking V2 public seam. Do not preserve unsafe V1 wrappers that
   manufacture empty authority or accept raw HIR.
3. Separate closed compilation from open editor analysis. An unresolved analysis
   result is never a compiled artifact.
4. Replace string-tagged relational stages with typed HIR variants.
5. Predicate references require call syntax; source rules are resolved as one
   document program before stratification.
6. Keep numeric semantics at finite `i128`/`f64` unless a separate language
   decision introduces arbitrary precision.
7. Planning remains host-agnostic semantic policy classification. Physical
   optimization, provider routing, execution, and SDK wiring stay out of repo.
8. Rename the current process and crate to
   `quanta-gqlang-v2-diagnostics-stdio`. Actual JSON-RPC/LSP support is a
   separate future packet. Ticket 08 rewrites the renamed service to bounded
   UTF-8 NDJSON; it remains neither JSON-RPC nor LSP.

## Wave Order

```text
01 proof/DAG foundation + mechanical transport rename
  ├─ 02 surface + typed HIR
  └─ 03 authority + valid-by-construction types/modules
          ↓
04 semantic kernel + document transaction
          ↓
05 relational algebra     ||     06 rule/graph logic
          ↓
07 semantic planning + curated facade
          ↓
08 front doors + resource/transport hardening
          ↓
09 semantic conformance + truth sync + closeout
```

## Non-Goals

1. runtime substrate
2. provider registry
3. physical operator selection or join-order optimization
4. backend cardinality statistics
5. semantica-owned request/response DTOs
6. SDK backend wiring
7. preserving source-incompatible V1 helper APIs

## Closure Rule

No ticket may claim completion from static source inspection, a routing smoke
test, or a focused unit run. Closure requires the ticket's RED cases, owner tests,
cross-stage tests, mutation case, exact command receipt, and clean scoped diff.

The packet is complete only when ticket 09 records a fresh exact-head `just ci`
receipt and every finding in `FINAL_AUDIT.md` is mapped to a landed owner change
or an explicit scope deletion.
