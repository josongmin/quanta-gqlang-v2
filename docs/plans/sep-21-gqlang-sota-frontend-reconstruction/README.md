# Sep 21 — QGLang SOTA++ Frontend Reconstruction

Status: `EXECUTION ACTIVE — OWNER CUTS ADVANCED; INTEGRATION AND CLOSEOUT RED`

## Authoritative Current Checkpoint — 2026-09-21

This section supersedes every older status sentence and test count below it.
Those sections remain only as audit provenance.

- frozen observation: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
- worktree: shared dirty overlay, 178 status entries at the final audit freeze
- immutable candidate: none
- packet status: not closed; no workspace, hosted-CI, or activation claim

| Ticket | Current state | Current proof | Exact remaining owner work |
|---|---|---|---|
| 00 | accepted scope | frontend-only boundary and source-reachability decisions remain authoritative | keep later tickets inside syntax/sema/IR/CLI/stdio ownership |
| 01 | receipt-scoped, not final | historical proof/DAG receipt; later architecture runs are RED | remove the remaining `sema -> surface` dev edge, refresh non-vacuous gates on the immutable candidate |
| 02 | owner implementation present | HIR/Surface historical owner suites; no current integrated receipt | preserve typed six-stage grammar and exact locations through the final pipeline/frontdoors; rerun on candidate |
| 03 | owner implementation present, delta unqualified | Types `89/89`, Modules `130/130`, Binder `54/54`; all three strict Clippy GREEN | add discriminating cross-owner capture/topology/row mutations, prove capture order independent of numeric node IDs, refresh public API inventories/hashes and combined frozen receipt |
| 04 | in progress | Sema all-target `87/87` GREEN; strict Sema Clippy GREEN; all root/Composition body families, Inline View topology, nested EXISTS, private local identity, local graph targets, and generated origins are implemented | repair implicit global aggregate Project and seal the downstream budget receipt; finish remaining transplant/public compile-fail rails and integrated receipt |
| 05 | in progress; SS-05B package-green | Algebra all-target `4/4` GREEN; strict Algebra Clippy GREEN; one sealed Sema ingress | remove the original EXISTS conjunct after Semi/Anti lowering; use sealed budget and iterative traversal; add Group/Aggregate/Sort/EXISTS operator and mutation matrix |
| 06 | planned/currently RED | reusable Tarjan/stratification code only; current Logic compile is broken by deleted Algebra V1 imports | add one Sema-document rule/graph adapter, iterative budgeted SCC, fixed aggregate policy, and delete the duplicate EXISTS producer |
| 07 | planned/currently RED | no current Pipeline/Planning/facade qualification | assemble one document artifact; make planning iterative and semantic-only; make facade a full API allowlist |
| 08 | planned | audit only | bounded wire DTO/framing/diagnostics/argv, shared facade consumer, deterministic exit/recovery policy, V1 deletion |
| 09 | closeout-only | audit receipt only; Semgrep is non-vacuous but RED at 7 rules / 95 targets / 13 findings | single manifest authority, base-to-head inventories, static-policy cleanup, exact-SHA local/hosted gates, separate tested SHA from attestation commit |

Current observed gates:

| Gate | Result | Meaning |
|---|---|---|
| `cargo test -p quanta-gqlang-v2-sema --all-targets` | GREEN, 87 tests | Ticket 04 owner behavior only |
| `cargo test -p quanta-gqlang-v2-algebra --all-targets` | GREEN, 4 tests | SS-05B boundary only; matrix incomplete |
| Types / Modules / Binder all-target | GREEN, 89 / 130 / 54 | current owner behavior; no combined immutable receipt |
| strict Types, Modules, Binder, Sema, Algebra Clippy | GREEN | scoped lint evidence only |
| `cargo fmt --all --check`; `git diff --check` | GREEN | current dirty-tree formatting only |
| architecture checker | RED: 20 normal transitions, one dev transition, plus illegal `sema -> surface` dev edge | transition and final DAG not closed |
| `cargo check -p quanta-gqlang-v2-pipeline --lib` | RED at Logic imports of deleted Algebra V1 types | consumer cutover not started/complete |
| workspace/final CI | unqualified | no immutable candidate or complete 01-09 receipt set |

Current critical path is:

```text
03 capture-proof and 04 aggregate-producer repairs
  -> 05 algebra owner matrix
  -> 06 logic semantic adapter
  -> 07 pipeline + planning + facade cut
  -> 08 frontdoors and resource bounds
  -> 09 immutable closeout
```

Consumer waves remain ordered, but each wave now has explicit disjoint writer
lanes and a serial integration barrier. See
[`WAVE_EXECUTION_PLAN.md`](./WAVE_EXECUTION_PLAN.md). Ticket 03 capture work and
Ticket 04 aggregate/budget work share one Sema writer; splitting those paths
would create patch-on-patch conflicts.

### Historical audit material below

The original base, intermediate commits, and older suite/error counts below
are retained to explain decisions. They are not current completion evidence.

Audit base: `main@e896a11024bd5a1e1ef03c094a70032f4e6c765f`

Audit date: `2026-09-21`

Execution started after the audit. The original audit base remains authoritative
for before/after comparisons. External integration commits `96eab779` and
`fcf106b` are recorded in `EXECUTION_LEDGER.md`; neither is a clean Ticket 03
verification boundary. Tickets 04-09 remain unverified, and only ticket 09 may
declare packet closure.

During final RR, externally created mixed-scope commits `ceb4142`, `6583091`,
and `f9ec954` advanced `main`/`origin`. They contain Ticket 03 plus binder,
algebra, and planning consumer deltas, but do not advance tickets 03-07 by
commit presence alone. Ticket 03 was subsequently owner-verified with an exact
dirty fingerprint and receipt; the binder/algebra/planning deltas remain
unqualified inputs to Tickets 04/05/07. `EXECUTION_LEDGER.md` records their
exact SHAs and audit status.

Ticket 04's live owner/consumer RR reopened Ticket 03 for types declaration/rule
proof and modules predeclaration/all-family reference authority. The earlier
Ticket 03 receipt remains valid only for its frozen owner scope; new producer
code requires a fresh receipt delta. Binder's 43 focused tests and strict
Clippy pass, but the sema transaction is not complete. The exact current
blockers and work packages are in Tickets 03/04 and `FINAL_AUDIT.md`.

The second live audit's all-target-proven sema paths are declarations and
`REQUIRE CERTIFIED` (8/8 focused transaction tests); a later GraphQuery test
and a SHOW Presentation test passed in isolation; later RuleBinding Catalog
and CHECK focused tests brought the dirty reachability checkpoint to 6/9;
a later Query/Source positive made that checkpoint 7/9 root variants, not
all-family proof. A subsequent call-free unary Rule path raises named root
reachability to 8/9; later source/host call-bearing Rule paths were added,
while Composition remains RED. Modules producer
predeclaration/host proof has 126 owner-local tests and strict Clippy on recorded
hashes; sema family integration is unproved. Types now has a later 85/85
owner-local suite and strict Clippy, while later Binder 47/47 and sema
52/52 owner runs remain partial; Binder strict Clippy passes but sema Clippy
is RED with 21 library and 43 library-test errors. These checkpoints do not change
Ticket 03/04 status.

During this final ticket audit, shared `main` and `origin/main` advanced to
`ecc38f59e9fdcd62b5e030445188ffa8b01e100a` in one external mixed
64-path commit. The original audit base and `f9ec954` owner checkpoints remain
historical; the remaining sema/docs dirty overlay and RED integration gates
are tracked in `EXECUTION_LEDGER.md`. Commit presence is not qualification.

Latest final-audit supersession on that HEAD plus the shared dirty overlay:
HIR 72/72, Surface 206/206, Binder 53/53, Types 86/86, Modules 126/126, and
Sema 59/59 all-target tests pass on the current dirty overlay; the latest
receipt still names the prior 56-test fingerprint. The strict Clippy lane
passes for the current Sema overlay.
The Sema count includes the independent checked-HIR Group candidate ancestry
and tamper oracle, so earlier statements that this exact oracle or Sema Clippy
remain open are historical. Composition, Inline VIEW, query relational EXISTS,
and the complete nine-family transaction remain open. Logic 49/49 and Planning
89/89 pass only on their reusable V1 contracts. Algebra's four suites fail to
compile with 74 retired-HIR call errors, Pipeline fails with 25 library and 29
library-test V1 import/shape errors, architecture remains in transition mode at 21 final + 22
transitional normal edges plus one transitional dev edge, and receipts 05–08
do not exist. The final block of each ticket is the authoritative current work
order when an earlier checkpoint differs.

The same final audit found `cargo fmt --all --check` RED on 27 files and final
architecture mode RED on 22 normal plus one dev transition edges. The passing
test/Clippy counts above are behavior/lint checkpoints, not complete receipts.
The shared tree was not bulk-formatted during the audit.

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
6. [WAVE_EXECUTION_PLAN.md](./WAVE_EXECUTION_PLAN.md) — canonical waves, path leases, barriers, and reopen rules

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
## 2026-09-21 Final Audit Checkpoint

The current tree is not releasable. Latest local owner evidence is Types 86,
Modules 126, Binder 53, and Sema 59 with strict Clippy GREEN; HIR 72, Surface
206, Logic 49, and Planning 89 also pass their direct suites. These results are
not an integrated receipt.

Integration remains RED: Algebra has 74 test compile errors, Pipeline has 25
library and 29 library-test compile errors, Conformance is blocked by Pipeline,
final architecture retains 22 normal plus one dev transition, and Rustfmt lists
27 unique files. Taplo and `git diff --check` are GREEN.

Types' deferred Composition rows and exact EXISTS captures are implemented and
Sema feeds them through one document-kernel call. Sema still rejects root
Composition, inline View, and Query EXISTS during final materialization. The
critical path is therefore unchanged: `03 topology proof -> 04 source closure
-> 05 -> 06 -> 07 -> 08 -> 09`. See each ticket's final current-tree audit for
the exact non-duplicating work sequence.
