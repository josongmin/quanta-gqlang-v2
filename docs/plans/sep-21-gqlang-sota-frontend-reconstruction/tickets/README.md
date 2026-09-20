# Ticket Index

Status: `EXECUTION ACTIVE — 01-02 FROZEN OWNER RECEIPTS, 01 DOC-GATE DELTA OPEN, 03 REPAIR, 04 IMPLEMENTING, 05-09 OPEN`

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

## Latest Live Work Order — `main@f9ec954` Plus Dirty Overlay

Read each ticket's final live-audit section for file-level steps. This table is
an ownership and no-duplicate-work index, not a replacement for its tests or
receipt. A dirty implementation is never an accepted landed output by itself.

| Ticket | Reuse; do not rebuild | Next unproved acceptance |
|---|---|---|
| 00 | frozen frontend/runtime and six-stage/12-op scope | keep the rule-body relational EXISTS semantic exclusion synchronized with 03/04 source and direct-solver tests |
| 01 | diagnostics-stdio rename, 14-member transition architecture gate and non-vacuous lint owner | retain historical receipt; execute the later doc-gate test on integrated source; final exception removal belongs to 09 |
| 02 | parser, formatter, direct six-stage HIR and integrity traversal | retain its scoped receipt; the later owned-document proof delta is 03/04 work, not a new syntax tree |
| 03 | one types kernel/registries and one modules validator/session; host key/proof owner code | prove sema host-call ingress/collision/SCC exclusion, remaining allocation/contextual negatives, mutation and fresh combined receipt; latest types 83/83 and modules 126/126 owner tests are scoped |
| 04 | Binder sequential frames/captures and sema checked DTO skeleton; latest sema 43/43 owner suite includes Source/Filter/Project/Join/Sort and host Rule call; later Group key-only Binder/sema focused 2+2 | produce all nine root families from source in one checked transaction; Group aggregate-only partition authorization, Composition, Inline VIEW, EXISTS/graph subqueries, and strict Clippy (45 errors at prior snapshot) remain RED |
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
