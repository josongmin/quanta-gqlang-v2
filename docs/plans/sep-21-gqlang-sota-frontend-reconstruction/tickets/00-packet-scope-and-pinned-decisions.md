# 00 — Packet Scope And Pinned Decisions

> 2026-09-23 execution-contract addendum: the frontend-only ownership rules below remain in force. The subsequent breaking V3 **execution** facade contract is specified in [`docs/contracts/frontend-execution-v3.md`](../../../contracts/frontend-execution-v3.md) and implemented under [Ticket 10a](./10a-v3-frontend-execution-contract.md). This does not reopen the V2 syntax/sema/planning cut or authorize runtime/provider/SDK ownership here.

Status: `SCOPE ACCEPTED; 01-09 REPO_LOCAL_CLOSED ON TESTED SOURCE 08e5ce5; SEMANTICA CONSUMER OPEN`

Current override (2026-09-24): the frontend-only pinned decisions below are
unchanged. [Receipt 09](../receipts/09.md) qualifies clean source
`08e5ce5ab05e8af141985d3b5f54bbff4a67e97f` locally; hosted CI is
unavailable and Semantica runtime remains downstream-owned. The following
`d9d0f28` checkpoint is historical.

Current source checkpoint: `main@d9d0f28a8365e6918dc8c7d0c552a8e277a838f0`.
The older execution order below is historical; do not redispatch it without a
new source-level reproducer. Ticket 09 owns remaining conformance/proof work,
and Ticket 10a owns the V3 producer contract. Semantica runtime remains outside
this repository.

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

## 2026-09-21 Owner/Consumer Audit Decisions

These refine the twenty frozen decisions without widening runtime ownership.
They supersede conflicting later sketches.

1. A closed graph-reference use requires an explicit immutable frontend symbol
   catalog with typed selector IDs/kinds and checked host registry identity.
   Graph selectors are not modules catalog asset IDs and are not a runtime
   graph-existence promise. Open analysis may omit that catalog only by
   returning explicit incomplete facts.
2. An unannotated unary rule head is admitted only as one Symbol. A closed
   source rule needs a types-owned unique concrete type proof for that variable
   and every quantified variable; an unanchored recursive rule is not a
   well-typed positive case. Sema does not choose a convenient type.
3. Source-less authority construction errors and source-backed document
   diagnostics are distinct variants of a frontend transaction error. No
   synthetic source ID or byte-zero span is used to disguise configuration
   failure.
4. Sema's typed source relation representation has the six variants already
   produced by HIR: Source, Filter, Project, Join (inner/cross subkind),
   GroupBy, and OrderBy. The existing 12-operator algebra adds only Apply,
   Aggregate, Limit, Union, SemiJoin, and AntiJoin beyond those stage families;
   it does not imply extra source syntax. `Unnest`, `Intersect`, and `Except`
   are not in the current algebra and are not authorized additions.
5. Document catalog reference kind mapping is explicit: View query reference
   to Query, Model source to ModelSource, Pack source to PackSource, Profile
   capability to Capability, Gate check and RuleBinding check to Check. These
   kinds are inert frontend metadata in modules; they do not introduce runtime
   provider or SDK ownership. Full authored catalog path spelling is retained
   under the selected module, with no punctuation-based ID guess.
6. The shared Boolean parser currently accepts relational `exists (from ...)`
   inside a rule body, but this packet admits relational existence only in
   query predicates. Ticket 04 must reject the rule-body form at its exact
   `RelationExists` span with `sema.invalid_item_shape`; Ticket 03's rule solver
   must also reject it explicitly if called directly. Add a source fixture and
   a direct solver fixture. This is a semantic prohibition, not a claim that
   the parser has separate rule/query Boolean grammars or that the form is
   supported in rules.
7. The breaking V2 semantic-cost success schema has four fixed qualitative
   dimensions: estimated cardinality class, recursion risk class, witness
   explosion risk class, and ordering contract. V1's fifth
   `unsupported_reason_code` is always `None` on successful plan producers;
   unsupported constructs return typed diagnostics instead. Ticket 07 removes
   the dead success field and puts reason codes only in failure diagnostics,
   with one updated planner→CLI schema rail. This preserves the prior D32
   requirement for a fixed host-agnostic schema, but deliberately changes its
   major-version field set. CPU/memory/I/O/network/coordination estimates are
   not frontend-owned.
8. `ORDER BY` has no authored null-placement syntax in this packet. Both
   ascending and descending keys default to `NULLS LAST`; sema seals that
   value in each checked order key, and algebra consumes it without choosing
   a second default. Changing this is a language-contract change, not a
   formatter or transport option.
9. `GROUP BY` publishes only its ordered key columns. Its input row remains a
   separate, non-public partition-input authority solely for the arguments
   of a types-proven aggregate call in a following projection. Ordinary
   filter/order/projection references cannot see dropped pre-group bindings.
   Binder records exact candidate identities but does not classify aggregate
   callees by spelling; sema authorizes those candidates from the one types
   callable/effects proof before the expression type walk.
10. Rule call spellings do not prefer source, builtin, or host. Any collision
    among the three exact authority domains is `sema.ambiguous_callable`.
    Modules `probe_reference(Predicate)` checks a potential host collision
    when a source or builtin candidate exists; host-only calls use the
    same-session `resolve_host_predicate_call` directly so a successful call
    is not double charged. Wrong-kind, ambiguity, and budget errors from
    modules remain hard errors.

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

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: `ACCEPTED`, with no new product decision and no implementation work in
this ticket. The latest source audit found no reason to widen the repository
beyond syntax, semantic authority, HIR, algebra/logic, planning metadata, CLI,
and diagnostics transport. Runtime plans, optimizer rewrites, provider
registries, SDK backends, JSON-RPC, and LSP remain out of scope.

Executor actions:

1. Preserve the six authored relation stages and exact 12 algebra target
   operators. Do not add syntax merely to make a target operator reachable.
2. Keep query relational `EXISTS` in scope and rule-body relational `EXISTS`
   rejected. Ticket 04 owns both source oracles; Ticket 05 owns Semi/Anti
   lowering; Ticket 06 must not reimplement either policy.
3. Preserve `NULLS LAST` for both sort directions, unary source rules,
   finite-`f64`/`i128`, and host-agnostic public contracts.
4. If an executor needs a different decision, stop before code changes and
   amend this ticket plus every affected downstream contract. A compatibility
   shim is not an amendment.

The unfinished-marker scan returned zero matches and `git diff --check`
exited 0. Ticket 09 must repeat both on an immutable candidate; a dirty-tree
result is not closeout evidence.

## Final Independent RR — Decision Ledger Action

No new product implementation is authorized here. The ten owner/consumer audit
decisions in this ticket are accepted as packet decisions 21–30. Update
`EXECUTION_LEDGER.md` to version and enumerate decisions 1–30; its current
“twenty decisions” acceptance text is stale and must not silently omit these
addenda. Ticket 09 verifies the exact decision count on the immutable candidate.
## 2026-09-21 Final Implementation-Overlap Audit

State remains `ACCEPTED`; this ticket owns decisions, not another compiler
implementation. Current-source review did not invalidate the frontend-only
boundary. In particular, numeric cardinality estimation, join-order search,
physical operators, provider wiring, and runtime execution remain explicitly
out of scope.

Workers must preserve these non-duplication rules:

1. Types owns expression/type/effect truth; Sema may only seal owner facts.
2. Modules owns catalog/session/topology authority; Sema may not resolve names
   with a second table.
3. Binder owns lexical identity/capture/order; Sema may not remint binding IDs.
4. Sema owns the checked document transaction and is the only input to Algebra.
5. Planning remains semantic policy/result classification, not a physical query
   optimizer.

No status advance is authorized until Tickets 03-09 produce their own receipts.

## SS-00F Current-State Contract Audit — 2026-09-21

This is the authoritative current-state addendum for Ticket 00. Audit start was
`main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` with 174 short-status entries.
The worktree was not clean, immutable, or attributable to one writer. No test
result in this packet may therefore be promoted to release or CI closure.

### Decision status

- The frontend-only boundary remains valid. Current source does not justify
  runtime execution, provider registry, physical operators, cardinality
  estimation, join-order search, JSON-RPC, or LSP ownership here.
- The single semantic path remains
  `Surface -> HIR -> Binder/Types/Modules -> Sema -> Algebra -> Logic -> Planning`.
  Algebra's new Sema-only production ingress confirms this direction; it does
  not authorize a second raw-HIR normalizer in Logic, Planning, or Pipeline.
- `SemanticDocumentV2` remains the only cross-item semantic document authority.
  Binder owns lexical identity/captures, Types owns type/effect/row truth,
  Modules owns catalog session/topology, and Sema only composes their proofs.
- The source grammar remains six relation stages. Algebra may expose the pinned
  twelve-family target vocabulary without inventing syntax to reach it.
- Historical receipt evidence remains historical; dirty-overlay results are
  checkpoints rather than release qualification.

### Remaining Ticket 00 work

Ticket 00 owns no production edit. Its only remaining closeout actions are:

1. After Tickets 03-08 stop changing public boundaries, compare every final
   public crate edge and API against these decisions; record a real decision
   change here before implementing it.
2. Ticket 09 freezes a clean candidate SHA and verifies that no public DTO can
   forge Binder, Types, Modules, Sema, or Algebra authority.
3. Reconcile the decision count and wording in `EXECUTION_LEDGER.md`; no worker
   may silently redefine a decision in a downstream ticket.

### Owner boundary and duplicate-work prohibition

- Do not add a compatibility API beside a retired API.
- Do not reconstruct Types rows/effects in Sema or Algebra.
- Do not rebuild Modules View topology in Sema.
- Do not infer Binder captures from names or Types traversal order.
- Do not put physical planning or runtime substrate behind the Planning name.

### Next SS / RR

`SS-00G` occurs only when the last public consumer cut is complete. It records
the candidate SHA, exact dirty state, final public edge set, and decision-ledger
hash. RR then checks every decision against actual source and rejects closure on
any compatibility shim, duplicated authority, undocumented scope change, or
dirty-tree-only proof.
