# 00 — Packet Scope And Pinned Decisions

Status: `ACCEPTED — EXECUTION ACTIVE`

## Goal

Freeze the architecture and language decisions that every later ticket relies
on. No production edit starts until these decisions are accepted as the packet
SSOT.

## Pinned Decisions

1. One new current-code `quanta-gqlang-v2-sema` owner is allowed.
2. The public cutover is a coordinated V2 break, not compatibility layering.
   Existing V1 implementations may remain frozen only inside the stacked 02-08
   integration branch; no adapter or release may expose both as supported APIs,
   and ticket 08 deletes V1 before integration.
3. Closed compile and open editor analysis return different result types.
4. Relation stages use typed HIR variants; string operators are removed.
5. Catalog/module/relation/predicate references resolve to typed IDs.
6. Source horn rules are compiled as one document program.
7. Predicate calls require call syntax; bare predicate names reject.
8. Numeric semantics are finite `i128`/`f64` for this packet.
9. Planning is semantic policy classification, not physical planning.
10. Rename the current flat-JSON crate and binary to
    `quanta-gqlang-v2-diagnostics-stdio`. Actual LSP is out of this packet and
    requires a separate JSON-RPC/document-lifecycle design.
11. Runtime, provider, SDK, and materialization ownership remain downstream.
12. Relational source grammar is pinned to `| join <source> on <predicate>` for
    inner join and `| cross join <source>` for predicate-free cross join.
    `| join <source>` rejects. This packet does not add UNION, LIMIT, HAVING,
    semi-join, or anti-join source syntax.
13. Existing correct algorithms are migrated in place or wrapped by one shared
    internal implementation during the cutover. A V2 type name does not
    authorize a copied binder, type checker, graph resolver, normalizer,
    Tarjan implementation, planner traversal, or transport test suite.
14. Boolean syntax distinguishes rule quantifiers from relational existence:
    `exists x, y in (<bool>)` and `forall x, y in (<bool>)` are rule-only;
    `exists (from ...)` and `not exists (from ...)` are relational subqueries.
    Query correlation may capture only aliases visible before that predicate.
    Zero-binder logical `exists (<bool>)` is removed as ambiguous.
15. HIR integrity is mandatory in the production document seam; `check_tree`
    is migrated into one document-wide proof, not documented as downstream-only.
16. Runtime-truth token policy has one Semgrep owner. The dead conformance
    denylist module is deleted after all 27 tokens and near-miss fixtures move.
17. The tracked example corpus is retained and receives a manifest-driven
    parse/format/compile/analyze/front-door rail.
18. Pipeline edge-case coverage retains one strengthened routing smoke; clause
    diagnostics remain surface-owned.
19. Test-only public accessors with no final production consumer are deleted
    with their sole-coverage tests; they are not treated as intentional API.
20. The ticket 01 sema crate is intentionally behavior-empty until ticket 04;
    architecture membership is its only ticket 01 rail.

## Public Success Definitions

`CompiledDocumentV2` means:

1. syntax integrity checked
2. all required authority present
3. every reference resolved and kind checked
4. every expression and relation typed
5. algebra/logic legality proved
6. global document constraints validated
7. semantic plan derived from sealed contracts

`AnalysisDocumentV2` may retain unresolved references but cannot be served or
converted implicitly to `CompiledDocumentV2`.

## Non-Goals

1. optimizer rewrites
2. numerical cardinality estimates
3. runtime execution plans
4. provider capability routing
5. arbitrary-precision numeric literals
6. backwards-compatible unchecked compilation

## Reject Or Escalate

1. preserving an old positive test whose success depended on skipped typing
2. placing semantic decisions in CLI, service, formatter, or planning
3. using empty context as a successful semantic authority
4. adding a second AST/IR stack outside the target architecture
5. copying a current algorithm into the new sema crate instead of calling or
   moving its canonical owner

## Executor Checklist

This ticket changes documentation only. The packet integrator performs these
steps before opening implementation branches:

1. Record `BASE_SHA=$(git rev-parse HEAD)` and `git status --short`.
2. Confirm the base is the SHA recorded in the packet README or update the audit
   delta before implementation.
3. Read every decision above and record `accepted` or an explicit replacement.
4. Confirm ticket 01 will mechanically rename the current crate/binary to
   `quanta-gqlang-v2-diagnostics-stdio`; ticket 08 hardens that transport and
   will not add JSON-RPC/LSP behavior.
5. Confirm rule arity stays unary for this packet.
6. Confirm `i128`/finite-`f64` numeric semantics. A different answer reopens
   tickets 02, 03, 04, 05, and 07.
7. Confirm that downstream consumers accept a breaking V2 facade. If not, stop;
   do not invent a compatibility wrapper inside an implementation ticket.
8. Allocate one owner per ticket and one integrator for reserved files.
9. Create a shared execution ledger containing ticket, owner, base SHA, state,
   write set, and dependency status.

## Required Outputs

1. frozen decision list with no unresolved decision markers
2. acknowledged `quanta-gqlang-v2-diagnostics-stdio` rename
3. named ticket owners and integrator
4. frozen base SHA and dirty fingerprint
5. acknowledgement that tickets 01-09 may not widen scope independently
6. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/EXECUTION_LEDGER.md`
   containing one row per ticket with owner, base SHA, state, dependency state,
   and write set

Execution record: [Sep-21 Frontend Reconstruction Execution Ledger](../EXECUTION_LEDGER.md).

## Verification

```bash
rg -n '\x54\x42\x44|\x54\x4f\x44\x4f|\x50\x4c\x41\x43\x45\x48\x4f\x4c\x44\x45\x52' \
  docs/plans/sep-21-gqlang-sota-frontend-reconstruction
git diff --check
```

The first command must return no matches. Unfinished-work markers are not
permitted in this packet; work is tracked by ticket status and explicit
remaining-work sections.

## Definition Of Done

1. all later tickets link to these decisions
2. no later ticket contains a contradictory owner or public API
3. any changed product decision updates this ticket before implementation
