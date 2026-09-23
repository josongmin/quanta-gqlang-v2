# Ticket Index

Breaking V3 frontend producer inventory and implementation: [Ticket 10a — V3 frontend execution contract](./10a-v3-frontend-execution-contract.md). [Ticket 10 — Downstream serving interop](./10-downstream-serving-interop.md) is historical cross-repository background; its V2 migration and old source pins are not the current producer contract. Neither ticket is qualified by the frontend V2 closeout receipts.

Status: `01-08 CODE INTEGRATED AT d9d0f28; CURRENT-SHA QUALIFICATION AND 09 CLOSEOUT OPEN; SEMANTICA CONSUMER OPEN`

## Current Routing — source `d9d0f28` (2026-09-23)

This table supersedes old `ecc38f59` and `3c8acbe` dispatch instructions
below. Before this documentation update, `main` and `origin/main` were clean at
`d9d0f28a8365e6918dc8c7d0c552a8e277a838f0` (tree
`9a2bcb4f9f64226d6539339c437922465f5fe483`). The matching pre-commit
source/test/tool overlay passed `just ci`, exit 0, nextest 1,168/1,168,
frontdoor parity 8/8. The latest clean exact-SHA receipt still belongs to
`3c8acbe` (1,102/1,102), not `d9d0f28`. None of this proves downstream
execution or packet-closeout attestation.
During this documentation update, another writer started uncommitted
Logic/Modules/Planning/Sema owner-test edits. They are in flight and excluded
from the `d9d0f28` gate. Re-freeze their final content before test-code
disposition or qualification; do not stage or overwrite them from this packet.

| Ticket | Current code/proof state | Remaining boundary |
|---|---|---|
| 00 | frontend-only boundary retained | no runtime/provider/SDK expansion |
| 01 | final architecture: 14 members, 26/26 normal and 5/5 dev edges; Semgrep and non-vacuity GREEN | historical ticket receipt remains source-bound |
| 02 | Surface/HIR implemented; source-depth and direct raw-HIR drop defenses committed | no second syntax/formatter authority |
| 03 | Types/Modules/Binder integrated in the 1,168-test pre-commit gate | historical owner mutation receipts are not new exact-SHA runs |
| 04 | Sema checked-document, F04 nested-aggregate rejection, F08 authored-order certification, composition, and budget paths committed | retain current semantic regression rails; do not resurrect old RED dispatch |
| 05 | Algebra normalization integrated | retain owner operator/EXISTS tests; no V1 fallback |
| 06 | Logic rule graph/stratification integrated | retain SCC, negation, and rule-program regression rails |
| 07 | Planning/Pipeline/facade integrated; external facade gate 1 positive, 15/15 negatives, 89/89 public roots | no runtime serving claim |
| 08 | CLI and diagnostics stdio integrated; frontdoor parity 8/8 in pre-commit gate | NDJSON is private diagnostics transport, not LSP |
| 09 | pre-commit full local gate GREEN; current clean-SHA qualification OPEN | audit existing semantic/example coverage, fill only proven gaps, then qualify current source and reconcile traceability/attestation; no hosted CI configured |
| 10a | V3 producer DTO/digests/typed literals and facade code integrated | qualify current source SHA; Semantica consumer is separate |

### Work remaining by acceptance layer

1. **Production code:** no previously listed W1-W6 implementation task remains
   open on `d9d0f28`. A new fix requires a current reproducer and its owner;
   historical RED prose is not a work order. V3 unsupported families remain
   an explicit admission boundary, not implicit executable payload.
2. **Test code:** `self_validation.rs` selects all 18 public roots, but the
   common loop checks broad kind/result/effect only. Compare retained option,
   reference, identity, and diagnostic assertions against existing owner
   tests; add only a demonstrated missing source-to-checked oracle. Do not
   fabricate an 18×11 matrix or second registry.
3. **Local proof:** candidate A `3c8acbe` has an exact SHA/tree receipt;
   `d9d0f28` has a passing pre-commit `just ci` on its source/test/tool
   content but no clean committed-HEAD `just qualify-local` receipt.
4. **Packet closeout:** Ticket 09 must first test its semantic/example claims
   against existing rails: `self_validation.rs` exercises all 18 registered
   families through the facade but checks broad disposition; Surface's
   `v3_examples_corpus.rs` already checks every shipped example against three
   docs and parse/format fixed points. Do not create a second manifest or
   inventory for the same facts. Fill a behavioral gap only when this audit
   finds one. Then reconcile traceability/status without pretending old owner
   mutations were rerun; a docs-only attestation B records `tested_sha=A`,
   `attestation_sha=B` and its diff.
5. **Downstream:** Ticket 10a producer is integrated; Semantica V3 admission,
   consumer, provider/generation, and result/receipt require a paired-repo
   proof. GitHub Actions is not configured and is not a local completion gate.

## Canonical Order

1. [00-packet-scope-and-pinned-decisions](./00-packet-scope-and-pinned-decisions.md)
2. [01-proof-rail-and-final-dag](./01-proof-rail-and-final-dag.md)
3. [02-surface-hir-semantic-shapes](./02-surface-hir-semantic-shapes.md)
4. [03-authority-types-modules-invalid-states](./03-authority-types-modules-invalid-states.md)
5. [04-sema-kernel-and-document-transaction](./04-sema-kernel-and-document-transaction.md)
6. [05-relational-algebra-and-aggregate-semantics](./05-relational-algebra-and-aggregate-semantics.md)
7. [06-rule-graph-logic-contracts](./06-rule-graph-logic-contracts.md)
8. [07-semantic-planning-and-public-facade](./07-semantic-planning-and-public-facade.md)
9. [08-frontdoors-transport-and-resource-bounds](./08-frontdoors-transport-and-resource-bounds.md)
10. [09-conformance-truth-sync-and-closeout](./09-conformance-truth-sync-and-closeout.md)

Worker protocol: [WORKER_START_HERE.md](./WORKER_START_HERE.md)

Canonical wave scheduler: [WAVE_EXECUTION_PLAN.md](../WAVE_EXECUTION_PLAN.md)

The final live-tree reconciliation is in the last section of Tickets 03-09 and
in [FINAL_AUDIT.md](../FINAL_AUDIT.md). It distinguishes existing owner
algorithms from unfinished cutover work; Ticket 01/02 receipts remain scoped
to their earlier owner proof, and neither `cargo check --lib` nor a focused
subset advances Ticket 03/04 to green.

## Dependency Matrix

| Ticket | Requires | May run in parallel with |
|---|---|---|
| 01 | 00 | none; integrator-owned foundation |
| 02 | 01 | 03 types RED preparation only |
| 03 | 01, and 02 before modules implementation | 02 only for types RED preparation |
| 04 | 02, 03 | none; central semantic cut |
| 05 | 04 | 06 with frozen sema contracts |
| 06 | 04 | 05 with frozen sema contracts |
| 07 | 05, 06 | none |
| 08 | 07 | none |
| 09 | 01-08 | none; closeout only |

## Per-Ticket Execution Contract

All paths are repository-relative. `new` means the ticket creates the file;
every other path must exist at the worker's frozen base. Run commands from the
repository root.

| Ticket | Start gate | Required landed output | Ticket-local acceptance |
|---|---|---|---|
| 00 | audited base and dirty state recorded | accepted decisions, owner ledger, unchanged scope | marker scan and `git diff --check` |
| 01 | 00 accepted | sema skeleton, mechanical diagnostics-stdio rename, final DAG plus exact transition ratchet, non-vacuous proof rails | all commands in 01 green with inspected and transition counts |
| 02 | 01 landed | typed source/HIR stages and UTF-8/formatter integrity | HIR/surface suites plus mutation cases |
| 03 | 01 landed; 02 location/HIR carrier frozen before modules changes | checked type/module/catalog/lock authority values | types/modules suites plus negative matrix |
| 04 | 02 and 03 landed | one document-scoped semantic transaction and internal semantic resolve/analyze APIs | sema + binder owner proof; pipeline failures deferred exactly |
| 05 | 04 V2 relation contract frozen on cutover stack | typed relational lowering over sealed aggregate/output contracts | algebra owner proof; pipeline failures deferred exactly |
| 06 | 04 V2 rule contract frozen on cutover stack | document rule program, stratification, static graph bounds | logic owner proof; pipeline failures deferred exactly |
| 07 | 05 and 06 focused green on cutover stack | final document assembly, semantic-only plan input, and curated public facade | planning/pipeline/facade suites and export scan |
| 08 | 07 focused green on cutover stack | shared front door, bounded source/framing/graphs, deterministic recovery, V1 deletion | CLI/diagnostics/pipeline budget suites plus workspace compile |
| 09 | immutable candidate contains 01-08 receipts | traceability, semantic family matrix, truth sync, exact-head receipt | complete final command list on one SHA |

## Latest Live Work Order — audited `main@f9ec954`, advanced to `main@ecc38f5` Plus Dirty Overlay

Read each ticket's final live-audit section for file-level steps. This table is
an ownership and no-duplicate-work index, not a replacement for its tests or
receipt. A dirty implementation is never an accepted landed output by itself.
The external mixed 64-path `ecc38f5` commit is recorded in the execution
ledger; it does not turn any owner-local checkpoint into Ticket 03/04 closure.

| Ticket | Reuse; do not rebuild | Next unproved acceptance |
|---|---|---|
| 00 | frozen frontend/runtime and six-stage/12-op scope | keep the rule-body relational EXISTS semantic exclusion synchronized with 03/04 source and direct-solver tests |
| 01 | diagnostics-stdio rename, 14-member transition architecture gate and non-vacuous lint owner | retain historical receipt; execute the later doc-gate test on integrated source; final exception removal belongs to 09 |
| 02 | parser, formatter, direct six-stage HIR and integrity traversal; exact `FROM` name-span delta now owner-green at HIR 72/72, surface 206/206 plus strict Clippy | retain the historical receipt and new dirty fingerprint separately; complete downstream integration without adding a second syntax tree |
| 03 | one types kernel/registries and one modules validator/session; host key/proof, intrinsic effects, deferred rows, and exact EXISTS capture inputs | add cross-owner capture-vector proof, remaining owner mutations, public API inventory, and frozen combined receipt; latest Types 87/87 and Modules 129/129 owner tests are scoped |
| 04 | Binder sequential frames/captures and sema checked DTO skeleton; Binder 53/53; current sema 59/59 plus strict Clippy (receipt remains at prior 56 fingerprint); exact `FROM`/graph ingress, Scalar aggregate reject, Group checked-HIR ancestry/tamper, non-key aggregate/collision, Rule polarity, and partial Composition/EXISTS type ingress have owner oracles | Sema must materialize all nine roots, Composition, Inline View, Query EXISTS/graph subforms, generated origins, and sealed identities; remove future-only dead-code expects when wired; refresh receipt |
| 05 | one 12-op algebra and existing owner suites | migrate raw-HIR normalizer to sealed six-stage relation input, preserve group/correlation, compile all four owner suites |
| 06 | existing Tarjan/stratifier and graph/witness checks | consume full typed rule set and sealed graph IDs; prove SCC/polarity and explicit transitive bounds from source |
| 07 | planning vocabulary/traversals | cover all nine semantic item kinds including Meta/Composition, preserve result/explain contracts, cut pipeline HIR/Binder/types edges, expose only curated facade; 89 V1 tests are not proof |
| 08 | renamed stdio byte loop, CLI and transport byte fixtures | use facade-only APIs, bounded NDJSON with CRLF lookbehind and inline/file byte caps, move parity to dependency-neutral two-binary script, delete frozen V1/LSP seams |
| 09 | conformance routing fixtures and artifact hygiene | prove disjoint 18-root+2-supplemental semantic manifests, truth/traceability inventories, final DAG and immutable committed-head gates; no closeout receipt exists |

A ticket is not ready if its start gate is missing, its RED cannot be observed,
or an intended path falls outside its write set. Amend the packet before coding;
do not make a local interpretation.

## State Machine

```text
PLANNED
  -> READY
  -> RED_CAPTURED
  -> IMPLEMENTING
  -> FOCUSED_GREEN
  -> INTEGRATED
  -> REPO_LOCAL_CLOSED
  -> CLOSED_BY_09
```

Only ticket 09 may set `REPO_LOCAL_CLOSED` or `CLOSED_BY_09`.
`REPO_LOCAL_CLOSED` requires one clean immutable candidate SHA and every local
gate but records hosted CI as unavailable; it is not remote closure. For tickets
04-06, `FOCUSED_GREEN`
explicitly means owner crates only and the receipt must enumerate deferred
consumer failures. Ticket 07 is the first cross-stage integration gate.
`FOCUSED_GREEN` never implies workspace, remote CI, or downstream closure.
Every state transition is recorded with SHA, dirty fingerprint, command, exit
code, selected/executed count, and owned-path diff.

Tickets 02-08 are a stacked breaking-API cutover because ticket 02 changes the
canonical raw HIR carriers consumed by later owners. Individual owner crates may
reach `FOCUSED_GREEN`, but no 02-07 intermediate may be released or marked
`INTEGRATED`. Ticket 08 migrates the frozen public inventory, proves owner-local
unchecked APIs were already removed by their owning tickets, runs the workspace
compile, and then advances 02-08 together.

## Reserved Integration Files

Only the integrator edits these during parallel work. The assigned ticket 01
foundation worker may edit items 1, 2, 6, and 7 for ticket 01 only; `/root`
reviews and integrates that scoped diff before any later ticket starts:

1. `/Cargo.toml`
2. `/Cargo.lock`
3. `/crates/quanta-gqlang-v2-pipeline/src/lib.rs`
4. `/crates/quanta-gqlang-v2-pipeline/src/compile.rs`
5. `/crates/quanta-gqlang-v2-frontend-contract/src/lib.rs`
6. `/tools/architecture/check_architecture.py`
7. `/.github/workflows/lint.yml`
8. packet status and closeout documents
9. `/crates/quanta-gqlang-v2-modules/Cargo.toml`
10. `/crates/quanta-gqlang-v2-algebra/Cargo.toml`
11. `/crates/quanta-gqlang-v2-logic/Cargo.toml`
12. `/crates/quanta-gqlang-v2-planning/Cargo.toml`
13. `/crates/quanta-gqlang-v2-cli/Cargo.toml`
14. `/crates/quanta-gqlang-v2-conformance/Cargo.toml`
15. `/crates/quanta-gqlang-v2-sema/Cargo.toml`
16. `/crates/quanta-gqlang-v2-pipeline/Cargo.toml`
17. `/crates/quanta-gqlang-v2-frontend-contract/Cargo.toml`
18. `/crates/quanta-gqlang-v2-diagnostics-stdio/Cargo.toml`

Workers may prepare a patch for a reserved file but do not merge or edit it in
the shared integration branch. The integrator applies the smallest required
change after checking the final DAG and all active write sets.

## Required Artifact Layout

Each implementation ticket stores its receipt at:

```text
docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/<ticket-id>.md
```

Ticket 01 creates the `receipts/` directory with `01.md`; each later ticket owns
only its numbered receipt. Ticket 09 fails closeout when any of `01.md` through
`08.md` is absent. Ticket 00's decision record remains its ticket file and
execution ledger.

## Ticket Receipt Schema

Every ticket closeout records:

```text
base_sha=
head_sha=
dirty_before=
dirty_after=
owned_files=
red_commands=
red_result=
green_commands=
green_result=
mutation_case=
cross_stage_receipt=
remaining_risks=
kept_assets=
migrated_assets=
deleted_assets=
new_assets=
existing_tests_migrated=
obsolete_tests_deleted=
duplicate_owner_scan=
```

`red_result` and `green_result` contain command, exit code, and test counts;
links to CI or prose such as `passed locally` are not substitutes. A mutation
case records the deliberate defect and the exact test that failed because of
it. Revert the mutation before producing `head_sha`.

Focused green is not packet closure. Ticket 09 owns the only full-workspace and
documentation closeout claim.

The asset/test fields are mandatory for tickets 01-09. `duplicate_owner_scan`
must include a source-symbol and test-file inventory demonstrating that a
migrated algorithm or suite has one active owner after the ticket. A statement
such as `no duplicates observed` without the command and inspected paths is not
a receipt.

## Final Audit Checkpoint — `main@ecc38f59` + Dirty Overlay

The final section of every Ticket 00–09 is the authoritative worker handoff for
current source. It records what to reuse, the implementation order, acceptance
commands, and stop conditions. Older sections remain historical rationale and
must not override a later count or gate.

- owner GREEN: HIR 72, Surface 206, Binder 53, Types 87, Modules 129, current Sema 64;
  their combined strict all-target Clippy lane exits 0;
- reusable V1 algorithm GREEN only: Logic 49, Planning 89;
- integration RED: Algebra four-suite compilation with 74 retired-HIR errors,
  and Pipeline compilation with 25 library/29 library-test V1 errors;
- proof rail GREEN only in transition mode: 21 final + 22 transitional normal
  edges and one transitional dev edge;
- closeout absent: receipts 05–08 and an immutable clean candidate do not exist.

The latest `cargo fmt --all --check` exits 0 and supersedes the earlier
23/27-file dirty-overlay format failures. This is current-tree format evidence,
not an immutable receipt. Final architecture mode remains RED on all 22 normal
and one dev transition edges.

Required serial critical path is `03 capture receipt -> 04 -> 05 -> 06 -> 07 -> 08 -> 09`. Tickets
05/06 may overlap only after Ticket 04 freezes the complete semantic contracts.
No worker may use a passing V1 owner count to skip the typed consumer cutover.

## Final Multi-Angle RR — Newly Pinned Blockers

The detailed owner work is in each ticket's final RR section. This index pins
the dependency-impacting deltas so executors do not rediscover or duplicate
them:

| Ticket | Newly pinned work | Duplicate work forbidden |
|---|---|---|
| 00 | ledger accepts decisions 1–30 | new scope registry |
| 01 | retain transition checker; later flip existing gate to final | second architecture checker |
| 02 | fingerprint/format the existing name-span delta and untracked test | second source-name span |
| 03 | Types-owned deferred multi-field Query row; exact neutral EXISTS captures; View topology-before-materialization | per-LET Types calls, Sema-created row types, second validator |
| 04 | sealed generated producer origins, checked scan uses, local graph targets, registry-complete closed carriers | raw Composition IDs, fake expression origins, host fallback |
| 05 | lower only frozen Sema relations/captures; account for all 74 retired-HIR errors | resolver/aggregate/capture DTO clones |
| 06 | adapt existing Tarjan/stratification/witness algorithms and delete final Logic→Algebra edge | second SCC/graph resolver |
| 07 | iterative shared algebra fold; structural compiled variants; full public API allowlist | recursive walkers, universal optional plan, undefined authority hash |
| 08 | current-head inventory; total bounded request/response/span/exit contract; output amplification caps | stale 244-line inventory, hidden transport caps |
| 09 | one manifest SSOT; recursive tracked hygiene; base-to-head inventory; both facade scripts in CI; two-SHA evidence | duplicate example registries, self-referential receipt SHA |

Critical path remains `03 producer delta -> 04 -> 05 -> 06 -> 07 -> 08 -> 09`.
Ticket 04 must not fake the missing Ticket 03 row/capture authority to preserve
the older `04 -> ...` diagram.
## 2026-09-21 Final Audit Routing

Current dirty-overlay checkpoint:

- 00 accepted;
- 01-02 receipt-scoped verified with unqualified deltas;
- 03 repair: Types 87/87, Modules 129/129, topology producer and Types-owned
  EXISTS effect/order implemented;
  exact capture authority, topology permutation/public-API mutations, Sema
  consumption, and frozen receipt remain open;
- 04 implementing: Binder 53/53, Sema 64/64, Query-only Composition source
  path implemented; non-Query Composition, Inline View, Query EXISTS and
  nine-root receipt remain open;
- 05 Algebra cutover open and currently 74 test compile errors;
- 06 Logic adapter open despite 49/49 direct tests;
- 07 public assembler/facade/Pipeline cutover open despite 89/89 Planning tests;
- 08 bounded front-door parity open;
- 09 final conformance/clean-candidate closeout open.

Read the last `Final Current-Tree Audit` section in the assigned ticket before
editing. It identifies existing owner implementations, duplicate-work stop
conditions, exact next work, and required proof. Historical green counts do not
override a later integration RED.

## SS-03E / SS-04E Routing Supersession — 2026-09-21

- Ticket 03 Types owner checkpoint is 88/88 plus strict Clippy GREEN. It proves
  exact earlier-row reuse, not topology-derived Inline View ordering.
- Ticket 04 Sema owner checkpoint is 66/66 plus strict Clippy GREEN. Query
  relational EXISTS now has sealed nested relation/correlation semantics.
- The next worker must take Inline View's single Modules-proof ->
  Types-pre-evaluation-order -> Sema-cache sequence; do not reopen Query EXISTS
  with a second relation materializer or start Tickets 05–09 early.

## SS-03F / SS-04F Routing Supersession — 2026-09-21

- Ticket 03 now has a validated relation pre-evaluation input/kernel checkpoint:
  89/89 Types tests and strict Clippy GREEN. It validates a projected order; it
  does not replace Modules topology authority.
- Ticket 04 now has a same-session Inline View checkpoint: 130/130 Modules and
  69/69 Sema tests, strict Clippy GREEN. The completed slice is direct View
  edge -> opaque Modules proof -> Types prewalk -> asset-keyed cache ->
  `family=None` semantic-edge equality -> finalization.
- Do not start Ticket 05 from this checkpoint. Ticket 04 remains implementing
  until the documented mutation/e2e/external-View/Composition obligations have
  a frozen receipt. Global diff check has one externally owned trailing-space
  failure; scoped changed-path diff check is GREEN.

## SS-04G Routing Supersession — 2026-09-21

- The raw Composition source tuple is retired. Any worker must consume the
  opaque `ResolvedCompositionSourceIdV2` carried by an exact Binder-derived
  binding; do not restore a coordinate constructor or name-keyed workaround.
- The next Ticket 04 worker owns non-Query Composition source producers/bodies
  and their authoritative row/capability contracts. It must extend the sealed
  transaction rather than branch around it.
- Ticket 05 remains blocked on the still-incomplete Ticket 04 semantic output
  and its required frozen receipt.

## SS-04H Routing Supersession — 2026-09-21

- Generated SAMPLE/FIND output must use `OutputColumnOriginV2::Generated` with
  its exact producer item. Do not restore a fake expression key to satisfy a
  relation-output constructor.
- The next implementation unit is the single Binder-first graph-target resolver
  and `Host | Composition` carrier. It must be shared by FIND, SAMPLE scope,
  CHECK, and EXPLAIN; separate local graph resolvers are prohibited.
