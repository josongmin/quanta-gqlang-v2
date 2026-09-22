# Wave Execution Plan

## Authority And Snapshot

This file is the canonical scheduler for the reconstruction packet. It owns
only wave order, path leases, barriers, and handoff rules. The final
current-state section in each ticket remains the implementation authority;
receipts own executed evidence; `EXECUTION_LEDGER.md` owns state transitions.

Planning snapshot: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`
plus the shared 178-entry dirty overlay observed on 2026-09-21. This is not a
candidate SHA. Re-freeze HEAD, status, and owned-path hashes at every dispatch.

## Scheduling Model

Use one integrator and at most three concurrent workers. A worker receives an
exclusive path lease, not a whole-ticket lease. Workers may read any repository
path but must not edit outside their lease. The integrator owns shared seams,
manifests, locks, architecture fixtures, receipts, packet indexes, and wave
barriers.

Until the dirty overlay is captured in a user-authorized checkpoint commit,
all workers operate in the shared workspace with non-overlapping path leases.
An isolated worktree created from `HEAD` does not contain the overlay and is not
an equivalent base. After a checkpoint commit, worktrees are allowed only when
every lane starts from the same exact SHA and records it.

Every implementation wave has four phases:

1. `Wn-I`: disjoint writer lanes implement against frozen producer inputs.
2. `Wn-R`: all writers stop; parallel read-only reviewers inspect ownership,
   semantic mutations, and dependency/public-API leakage on one fingerprint.
3. `Wn-Q`: frozen-source test/Clippy runners execute in parallel with separate
   `CARGO_TARGET_DIR` values. A runner verifies its frozen inputs before and
   after the command.
4. `Wn-X`: the integrator alone applies reserved-path changes, runs shared
   gates, writes the receipt, and opens the next wave.

The exact acceptance-command list in each ticket's final current-state section
is command authority. The scheduler may split those commands among frozen-
source runners but may not omit or weaken them.

### Integrator-reserved paths

These paths are integrator-only unless a wave explicitly grants one named
sole-writer override. An override never permits two concurrent writers.

- root `Cargo.toml` and `Cargo.lock`;
- `.github/workflows/lint.yml` and `Justfile`;
- `tools/architecture/check_architecture.py` and
  `tools/architecture/tests/test_architecture.py`;
- `crates/*/Cargo.toml` whenever an edge changes;
- `crates/quanta-gqlang-v2-*/src/lib.rs` when it changes public exports;
- `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/{README.md,FINAL_AUDIT.md,IMPLEMENTATION_REUSE_AUDIT.md,EXECUTION_LEDGER.md,WAVE_EXECUTION_PLAN.md}`;
- `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/tickets/{README.md,WORKER_START_HERE.md}`;
- every final receipt and any cross-ticket fixture named below.

Workers may submit a proposed patch for a reserved path, but only the
integrator applies it after all writer lanes stop.

### Universal lane contract

Every dispatch must include:

```text
wave=
lane=
work_package=
base_sha=
overlay_status_hash=
allowed_write_paths=
integrator_reserved_paths=
producer_contract_hashes=
frozen_input_paths_and_hashes=
expected_red_command_and_result=
required_positive_and_negative_oracles=
required_deliberate_mutations=
acceptance_commands=
receipt_path=
deferred_consumer_failures=
```

The lane stops immediately if it needs an unleased path, an upstream public
contract changes, or one of its declared frozen inputs changes. Unrelated
parallel paths may change without invalidating a lane-local proof; workspace,
wave, and candidate receipts still require global quiescence. A lane does not
rebase, resolve a shared-file conflict, add a V1 alias, invent a default
budget, or bypass the sealed semantic document.

Adversarial input fixtures belong in normal tests and may run in the shared
workspace. A temporary source-code mutation must not run there. Source
mutation/revert proof requires a user-authorized local checkpoint commit and an
isolated worktree per mutation lane; otherwise record the proof as deferred.

## Wave Status

| Wave | Scope | Current state | Maximum useful parallelism |
|---|---|---|---|
| W0 | freeze and lease | `GREEN` (see ledger W0 record) | integrator only |
| W1 | 02 inventory + 03 authority proof + 04 Sema producer closure | `GREEN` (barrier record + checkpoint commit; receipts 02/03/04 closed) | 3 workers + integrator barrier |
| W2 | Algebra closure | `GREEN` (barrier record + checkpoint `4073b64`; Receipt 05 closed) | 2 workers after contract freeze |
| W3 | Logic closure | `GREEN` (barrier record; Receipt 06 closed; deps cut to Sema+Types) | 3 workers after Logic internal-interface freeze |
| W4 | Planning + Pipeline + facade | `GREEN` (barrier record; Receipt 07 closed; planning 137, pipeline 30, facade gate PASS) | 2 workers, then serial facade |
| W4 | Planning + Pipeline + facade | `BLOCKED BY W3` | 2 workers, then serial facade |
| W5 | CLI + diagnostics stdio | `READY` (entrance met: Receipt 07 GREEN; F-02 deleted-API imports are W5's first task) | 2 workers, then serial parity integration |
| W6 | conformance + hygiene + static debt | `PARTIAL PREP ALLOWED` | 3 workers; semantic execution waits for W5 |
| W7 | candidate qualification and attestation | `BLOCKED BY W1-W6` | strictly serial |

Critical path:

```text
W0
 -> W1 {SS-02G || SS-03H || SS-04W}
 -> W2 SS-05C
 -> W3 SS-06B
 -> W4 SS-07B
 -> W5 SS-08B
 -> W6 SS-09B implementation gates
 -> W7 candidate A -> attestation B
```

W6 schema/generator work may run early, but no generated truth output or
closure receipt is valid before W5 freezes the public facade and front doors.

## W0 — Freeze, Conflict Census, And Leases

### Integrator work

1. Record `git rev-parse HEAD`, `git status --short`, status-list hash, and
   SHA-256 for every leased existing file.
2. Record pre-existing dirty ownership. The staged external deletion
   `test_repro_cases.txt` remains outside this packet and must not be restored,
   committed, or absorbed.
3. Reproduce the current first RED: Logic compilation at deleted Algebra V1
   imports. Record architecture final-mode, Semgrep, and allow-attribute counts
   as baseline only.
4. Publish the active lease table in the execution ledger before workers edit.

### Barrier W0

W0 passes only when every lane has exact allowed paths, reserved paths,
producer hashes, and a named RED. An ambiguous directory-wide lease is invalid
when another lane owns a file below that directory.

## W1 — Upstream Authority And Sema Producer Closure

W1 has three worker slots. If more work is available, queue it behind the same
lease rather than adding a fourth concurrent writer.

### Lane W1-A — SS-02G inventory and receipt preparation

- **Write lease:** new
  `receipts/02-public-api-inventory.md` only; Receipt 02 remains integrator-
  reserved.
- **Read-only source:** HIR and Surface crates.
- **Work:** inventory the single
  `relation_source(name, name_span, alias, span)` path, absence of compatibility
  overloads, the untracked name-span test's intended enrollment, and exact
  owned-path hashes. Product source remains read-only.
- **Done:** public API inventory plus owner-path hash manifest and no claimed
  implementation delta. If the required public-API tool is unavailable, record
  the exact command/error as `BLOCKED_TOOLING`; do not substitute a text scan.

### Lane W1-B — SS-03H Types/Modules owner mutations

- **Write lease:**
  `types/tests/v3_types_check_edges.rs` and
  `modules/tests/v3_document_predeclaration.rs` only.
- **Reserved:** all Sema files, Binder files, public `lib.rs` exports, Receipt
  03, and the cross-owner transaction fixture.
- **Work:** three-producer pre-evaluation transplants and plausible swapped
  targets; topology diamond plus unrelated ready View across asset/edge
  permutations; exact diagnostic/location assertions; Types/Modules public API
  inventory draft.
- **Done:** Types and Modules owner suites and strict Clippy pass; each named
  adversarial fixture is rejected; no production source change and no new
  public forgery constructor. If a fixture exposes an owner defect, stop and
  amend the lease before repairing source.

### Lane W1-C — SS-03H-S + SS-04W combined Sema owner

- **Write lease:** `sema/src/{authority,document,relation,diagnostic,lib}.rs`,
  `sema/tests/v3_document_transaction.rs`, and
  `sema/tests/v3_semantic_budget.rs`.
- **Reserved:** Sema manifest, Receipts 03/04, and packet docs. Any other Sema
  path requires a lease amendment before editing.
- **Work:** first extract a private capture-vector equality validator and add
  dropped/duplicate/reversed/wrong-owner/wrong-ordinal/foreign/wrong-root/
  empty-set-omission cases. Keep validator-internal forgery tests under
  `document.rs` `cfg(test)` rather than exposing a public mutation API. Add the
  one-document, exactly-once Types-kernel invocation oracle. Then produce the
  private scalar `Group(empty)`, preserve grouped partition identity, seal
  aggregate output invariants, and retain one immutable downstream budget
  receipt in `SemanticDocumentV2`.
- **Budget freeze:** the receipt must cover Algebra relation/expression steps,
  Logic rule nodes/edges/relaxation/witness depth/count, Planning traversal,
  source/frame bytes, `max_diagnostics`, and `max_response_bytes`. This is the
  only budget authority. Tickets 05, 06, 07, and 08 may project or consume it,
  never extend it or accept caller-selected replacements.
- **Done:** scalar/grouped aggregate exact trees, star-negative matrix, budget
  identity/value mutation, and exact-limit/+1 pass locally.

### W1 integrator barrier — SS-03H/SS-04W cross-owner closure

After all lanes stop, the integrator alone:

1. reviews the combined Sema lane against both Tickets 03 and 04 rather than
   applying a second Sema patch;
2. applies only required manifest changes;
3. reconciles the Sema budget carrier with every downstream projection;
4. freezes all source writers, then runs HIR/Surface, Binder/Types/Modules, and
   Sema qualifications in parallel with
   `CARGO_TARGET_DIR=/tmp/qg-wave-w1-q02`,
   `/tmp/qg-wave-w1-q03`, and `/tmp/qg-wave-w1-q04` respectively;
5. runs scoped format/diff once, then writes Receipts 02, 03, and 04 on one
   unchanged fingerprint.

W1 does not pass on test counts alone. RR-04W must inspect the concrete
`Group(empty) -> Project(aggregate)` and grouped partition-retention shapes.
Any missing downstream budget dimension reopens W1 before Algebra begins.

## W2 — Algebra Closure

Entrance gate: W1 receipts freeze the aggregate-ready semantic document and
the complete budget receipt. Record hashes for the Sema public DTOs and do not
edit them in W2.

### Lane W2-A — Algebra lowering implementation

- **Write lease:** `algebra/src/{normalize,node,diagnostic}.rs` and private unit
  tests in those modules.
- **Reserved:** `algebra/src/lib.rs`, Algebra manifest, integration tests,
  Receipt 05, and architecture fixtures.
- **Work:** remove residual EXISTS predicates after Semi/Anti lowering; retain
  ordinary conjunctions only; replace recursive relation/expression walks with
  budgeted iterative frames; strengthen checked node/output invariants; either
  land checked Apply/Limit/Union constructors with a real consumer contract or
  remove them from the completion claim.

### Lane W2-B — Algebra adversarial matrix

- **Write lease:** `algebra/tests/*.rs` except `RETIRED_LAWS.md` until the
  integrator decides each archived law's disposition.
- **Reserved:** all Algebra source and public exports.
- **Start gate inside W2:** W2-A has frozen its private constructor/test-support
  shape. Until then this lane may only inventory existing tests and design
  fixtures; it must not chase a moving source API.
- **Work:** Project, Join, Group, Aggregate, Sort, EXISTS/correlation, deep-tree
  exact/+1, and forged identity/output/cardinality mutations against the frozen
  Sema DTO. It may not repair source.

### Optional lane W2-C — read-only Logic migration inventory

- **Write lease:** none in product source; lane-local report only.
- **Work:** map every V1 Logic source/test case to keep, migrate, or delete and
  propose the W3 path split. It must not restore deleted Algebra symbols.

### Barrier W2

Integrator applies `lib.rs`/manifest changes, reconciles `RETIRED_LAWS.md`, freezes
source writers, runs
Sema + Algebra tests, strict Algebra Clippy, scoped format/diff, architecture,
and deliberate mutation recovery. Receipt 05 must prove pure EXISTS has no
residual Filter and mixed conjunction retains only ordinary predicates.

## W3 — Logic Closure

Entrance gate: Receipt 05 is owner-GREEN and Logic's only intended source
authority is the frozen Sema document. The integrator first freezes private
interfaces between rule projection, SCC analysis, graph validation, and
witness construction; then the lanes start.

### Lane W3-A — Rule program and SCC

- **Write lease:** `logic/src/{recursion,aggregate}.rs` and rule/SCC-focused
  tests.
- **Work:** document-member stable IDs, iterative budgeted Tarjan and
  relaxation, canonical cycle witness selection, and reject-all recursive
  aggregates. Prove a 50k-deep chain without process-stack recursion.

### Lane W3-B — Graph contract and witnesses

- **Write lease:** `logic/src/{graph,witness}.rs` and graph/witness-focused
  tests.
- **Work:** consume resolved graph queries, require positive transitive depth,
  retain via/through order, accept depth plus waypoints, remove PointsTo and
  default-cap shortcuts, and charge all witness work before growth.

### Lane W3-C — Sema source adapter and V1 deletion

- **Write lease:** `logic/src/source.rs` and source-adapter tests.
- **Reserved:** `logic/src/{lib,diagnostic}.rs`, Logic manifest, lockfile,
  architecture fixtures, and Receipt 06.
- **Work:** build one `RuleProgramContractV2::from_document`, validate complete
  membership/edge/location identity, and delete the duplicate EXISTS lowerer.
  This lane does not add transitional adapters.

### Barrier W3

Integrator owns shared diagnostic/export integration, removes Logic ->
Algebra/HIR edges, updates architecture expectations, and runs Logic all-target
tests, strict Clippy, exact/+1 mutations, final dependency scans, and Receipt
06. The final normal Logic dependencies are Sema + Types only.

## W4 — Planning, Pipeline, And Public Facade

Entrance gate: Logic is GREEN and the compiled-document DTO skeleton, semantic
diagnostic ordering, and budget projections are frozen.

### Lane W4-A — Planning

- **Write lease:** Planning `src/*.rs` except public `lib.rs`, plus Planning
  tests.
- **Work:** replace `PlanRequestV1`/classification with exhaustive sealed-input
  dispatch; remove caller-selected truth/result/effect; merge recursive walks
  into one deterministic iterative budgeted traversal over checked Algebra.

### Lane W4-B — Pipeline

- **Write lease:** `pipeline/src/{compile,diagnostic}.rs` and Pipeline tests.
- **Work:** replace the V1 compiler body with Surface -> Sema -> Algebra ->
  Logic -> Planning orchestration; assemble all results by exact semantic keys;
  reject missing/extra/duplicate/foreign/wrong-kind and composition/rule/graph
  substitutions; remove raw HIR/Binder/Types/Modules authority logic.

### Lane W4-C — read-only facade consumer design

- **Write lease:** an isolated new fixture directory under `tools/` only after
  the integrator assigns its exact path; no facade product edit.
- **Work:** prepare the external-consumer compile allowlist and negative cases
  against the frozen DTO names. It may not depend on owner crates.

### Barrier W4

The integrator applies Planning/Pipeline manifests and exports, then serially
replaces facade globs in `frontend-contract/src/lib.rs`. The facade normal-
depends only on Pipeline and exposes the three pinned source-document functions
plus an explicit DTO allowlist. Run Planning, Pipeline, facade, docs, external
consumer, strict Clippy, and architecture `--require-final`. Write Receipt 07.
Ticket 08 starts only after facade DTOs and diagnostic ordering are frozen.

## W5 — Bounded Front Doors

Entrance gate: Receipt 07 is GREEN and W1 already owns
`max_diagnostics`/`max_response_bytes`. If those fields are missing, reopen W1;
do not amend the budget in a front-door patch.

Before the worker lanes start, the integrator changes and freezes the CLI and
diagnostics-stdio manifests so both compile only against `frontend-contract`.
The workers must not temporarily retain internal owner dependencies.

### Lane W5-A — CLI

- **Write lease:** CLI source/tests except manifest changes.
- **Work:** depend only on the facade; bounded file/pipe/special-file read with
  limit+1; checked argv UTF-8 byte accumulation before clone/join; exact exit
  mapping; no source-byte echo on errors; remove active LSP naming.

### Lane W5-B — diagnostics stdio

- **Write lease:** diagnostics-stdio source/tests except manifest changes.
- **Work:** byte-oriented CRLF-aware bounded NDJSON state, one response per
  line, suffix-preserving recovery, maintained JSON parser with duplicate-key
  rejection, total request/response schema, process failure latch, short-write
  and one-byte-reader proofs; remove handwritten flat JSON and LSP naming.

### Lane W5-C — parity harness preparation

- **Write lease:** `tools/test_frontdoor_parity.py` and isolated parity fixture
  data only.
- **Reserved:** CLI/stdio product source, Cargo files, shared docs, Justfile,
  workflow, conformance tests, and Receipt 08.
- **Work:** define one corpus and canonical comparison of facade, CLI, and
  stdio results/diagnostics/status. Execute only after W5-A/B freeze.

### Barrier W5

Integrator applies dependency edges and shared DTO/docs updates, runs both
front-door suites, conformance parity, strict Clippy, parity script, legacy-LSP
inventory, architecture final mode, and Receipt 08. Exact/+1 is required for
payload, source, diagnostic count, and response bytes; recovery must preserve a
following valid frame.

## W6 — Conformance, Hygiene, And Static Policy

Candidate-independent schema/generator work may be prepared earlier in new,
disjoint paths. It must not generate current truth, modify CI, or claim closure
until W5 passes.

Before semantic W6 workers start, the integrator migrates and freezes the
conformance manifest to the facade dependency. No lane may restore owner-crate
dependencies for fixture convenience.

### Lane W6-A — Semantic manifest and conformance

- **Write lease:** conformance fixture manifest and the three new V2 test
  targets named by Ticket 09; conformance product support only when isolated
  from W6-B/C.
- **Work:** one 18-example manifest plus two-row Decl/HornRule supplement;
  sealed-facade execution; declared/selected/executed counts; exact payload or
  diagnostic/span/resource/mutation/receipt assertions.

### Lane W6-B — Recursive hygiene and inventory generators

- **Write lease:** new generator/validator tools and their tests only.
- **Work:** derive tracked paths and `PACKET_BASE_SHA..candidate` changes,
  including deletes and both sides of renames; reject nested artifacts,
  duplicates, stale anchors, orphan crates, and unexplained dirt. Generate to a
  temporary directory until the integrator freezes inputs.

### Lane W6-C — Static-debt owner routing

- **Write lease:** none in the repository; return a lane-local audit report.
  Product findings are returned to the owning prior-wave crate and cannot be
  bulk-edited across owners.
- **Work:** map every current Semgrep finding and allow attribute to a concrete
  owner disposition; prove Semgrep rule non-vacuity; propose justified
  `#[expect(..., reason=...)]` only where removal is incorrect.

### Barrier W6

Integrator freezes the public source, runs generators once, applies truth docs,
adds external-consumer/parity/final-architecture/static gates to Justfile and
CI, and verifies both directions of every inventory. Receipt 09 may advance to
candidate-ready only when Semgrep is zero, all allows are removed or justified,
MSRV/current strict Clippy agree, and final architecture has no transition
edge. No lane may touch source after this barrier.

## W7 — Candidate A And Attestation B

W7 is strictly serial and requires explicit user authorization before commits.

1. Re-freeze status and prove only intended packet paths remain.
2. Create candidate A containing implementation, generated truth, and an
   unclaimed receipt template.
3. Run Ticket 09's mandatory command sequence on exact SHA A without any source
   or config mutation. Record exit code and selected/executed count for every
   command.
4. Query hosted CI independently for exact SHA A. `failed` blocks; queued or
   running remains open; unavailable permits only `REPO_LOCAL_CLOSED`.
5. Create docs/receipt-only attestation B and prove `A..B` contains no source,
   manifest, workflow, tool, or generated-truth change.
6. Rerun claim/diff/format/doc/inventory validators on B. Record
   `tested_sha=A` and `attestation_sha=B`.

No worker lane exists in W7. Any source change invalidates candidate A and
returns scheduling to the owning earlier wave.

## Failure And Reopen Rules

- **Producer contract insufficient:** stop the consumer and reopen the owning
  earlier wave. Do not patch the consumer around it.
- **Producer hash changed:** invalidate every downstream lane receipt built on
  the old hash, re-freeze, and rerun its RED/acceptance sequence.
- **Reserved-path conflict:** both workers stop; the integrator resolves after
  comparing pre-edit hashes. Workers never choose a winner.
- **Test-only lane discovers source defect:** report the exact failing mutation
  to the source lane; do not edit source under a test lease.
- **Architecture needs a temporary reverse edge:** reject the patch. The wave
  is not complete enough to integrate.
- **Budget field missing downstream:** reopen W1. No local defaults, optional
  budget, hardcoded caps, or environment-selected values.
- **Qualification mutates files:** the run is invalid. Restore only the
  command-generated files using explicit paths, repair the owner, and restart
  from a new candidate.

## Integrator Wave Handoff Record

Append one record per barrier to `EXECUTION_LEDGER.md`:

```text
wave=
entry_sha=
entry_status_hash=
lane_ids=
leased_paths_and_pre_hashes=
producer_contract_hashes=
lane_red_evidence=
lane_acceptance_commands_and_counts=
mutation_ids_killed_and_reverted=
reserved_path_changes_applied_by_integrator=
exit_sha=
exit_status_hash=
receipt_paths=
deferred_failures=
next_wave_gate=
```

A wave is `GREEN` only when all lanes and the serial barrier are GREEN on one
unchanged fingerprint. Parallel lane success without barrier integration is
`LANES_GREEN, WAVE_OPEN`, never ticket closure.
