# Parallel closeout dispatch

Updated: 2026-10-08. Initial source audit: Semantica
`871224b2059422a8f36eca91ffb0c9119238ac9c`, GQLang code input
`01cfe39ca0571b4fc099d41b24628fda70e754fe`. Current execution attempts and accepted
scopes are in the [active handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#r60r63-source-stability-and-clean-candidate).
Refreshed source/claims are required
before writing. This document is an execution plan; no implementation or runtime
result is claimed by its publication.

[Current ticket states](../../work/remaining.md), [scope and DoD](tickets.md),
[code boundaries](code-map.md) and [test requests](test-plan.md) remain authoritative
for their own subjects. This dispatch adds no new defect queue or engine project.

## Objective and priority

Finish existing canonical-owner migrations and budget/output custody, remove
demonstrated duplicate accounting mechanics, and prove the real consumers on a
single accepted source tuple. Use the current owners and interfaces.

1. **P0:** accept the integrated checked-read owners; complete output/binding/
   resource custody through adapter→context→extractors.
2. **P1:** real SDK deadline tests, integrated callback/cursor/language acceptance,
   D1–D7 and added SCC callers, affected policies and actual Native/product proof.
3. **Resolved:** QG-DIAG keeps the private primary-only wire required by the actual
   parity/NDJSON consumers; ADR-0008 records the decision. No new wire work is queued.
4. **External:** signed installed Linux CHECK trust. Inventory prerequisites early;
   its unavailability leaves FCP-INSTALL open without preventing local owner work.

All inventoried plugins and DefUse source cuts are integrated, including Python
`27d9505015e`; the production retired-carrier census is0. TypeScript joins C/Go's
snapshot/control cut, not a new
read-migration assignment. Proposed A-series query/optimizer work remains separate.

## Four slots and existing writers

| Slot | Accountability | Write constraint |
| --- | --- | --- |
| I — integrator | Source/claims freeze, shared-signature agreement, independent review, serial main checkpoints, proof receipts and ticket states | Only reviewed complete bundles; no blanket staging or resetting foreign work |
| A — existing facts owner | Integrated Python acceptance, then common DefUse/output producer handoff | Reuse the existing D0 facts/context writer; another agent reviews while that writer owns the cut |
| B — independent consumer/test owner | SDK deadline tests first, then C/Go/TypeScript snapshot control consumers | Consumer files only; shared contract/module registration goes through A |
| C — independent binding/proof owner | Kotlin admission/oracles, then SCC caller and policy request review | Consumer files only; no shared issuer/context writer |

This is a ceiling of one integrator plus three workers, not four fresh workers
in addition to the active Python/shared owners. Existing writers occupy the
corresponding slot; a review-only agent can audit their result. Refresh external
claims/barriers and reconcile actual capacity before dispatch. If a required
owner is busy, use the slot for independent request/oracle review until handoff.

## Wave schedule

| Wave | I | A | B | C | Exit / checkpoint |
| --- | --- | --- | --- | --- | --- |
| W0 — ownership and design | Record HEAD/tree/dirty/claims, affected caller closure and shared signatures | Existing Python/facts writer identifies complete bundle; trace actual DFG source/operation issuer and output retainer | Confirm real SDK caller and deterministic deadline oracles; inspect three snapshot ledgers | Review Kotlin exact source/backend contract and eleven SCC coverage scopes | Real DFG authority witness plus exact paths/symbols, lifetime, compatibility and rejection; no overlapping shared writer |
| W1 — independent completion | Review and integrate complete independent bundles serially | Accept committed Python checked extraction and binding tests | Review authored real fluent SDK timeout0/domain/value tests; execution pending | Kotlin binding is committed `df315e6` with exact1/whole70 owner passes; QG-DIAG is resolved by ADR-0008 | CP1 Python source integrated; CP2 SDK test bundle pending actual execution. Original Kotlin output custody still needs the agreed admission interface |
| W2 — coupled resource repair | Verify producer→all consumers→final retainer; integrate one complete contract bundle | Existing steward implements missing output/context/source-admission seam and actual original issuer/retainer wiring | Shared local counter integrated `f2d85a9` with2 owner passes; finish C/Go/TypeScript original birth, post-snapshot work/output | Wire Kotlin exact Native binding/current/output and tests to the same admitted contract | CP3 coupled producer + consumer compatibility; all signatures/callers compile and independent resource oracles pass |
| W3 — owner map and local proof | Freeze committed tuple and run admitted heavy owner jobs serially | Review all changed language, callback, original5 and custody witnesses/receipts | Reconcile D1–D7 feature classes and accepted historical inputs; include latest Shape/Cost projection | Reconcile eleven SCC caller scopes, affected policies and product/install prerequisites | CP4 current affected owner + feature/policy verdicts; only reproduced failures reopen source |
| W4 — real product closeout | Run/attest current product build, SDK/Native and process owners on same tuple | Review original source/first-refusal/custody closure | Review actual SDK/Preview/Graph result oracle and cleanup counts | Verify release-owned installed trust, then installed CHECK when available | CP5 process proof; CP6 installed proof if prerequisites exist; close only accepted ticket rows |

W1 source/test authoring can run independently; Rust execution remains subject to
normal lane/cache/resource admission. At this campaign's default, one heavy
logical build lane runs at a time. Read reviews and unconflicted test authoring
can continue while the lane builds. A module filter does not reduce `lib(test)`
compile cost. Kotlin and scalar consumers may proceed with a reviewed stable
signature, but the public contract bundle integrates together.

W2 is the critical shared dependency. Do not block SDK test authoring, D1/D2 map
review or F2 minimal registry review behind unrelated resource/solver work.
Never leave dependent callers uncompilable at a main checkpoint.

## Exact write-set handoffs

Paths below are Semantica-relative; `core/` means `packages/core/`, `qv2/` means
`packages/analysis/quanta-v2/crates/`. Refresh existing ownership before editing.

### A — single shared steward and existing Python owner

- Existing Python: `core/codegraph-defuse-extractor-python/src/extractor/` reachable
  checked frontdoor, binding/callable/assignment/traversal/projection helpers and
  specifically handed-off tests. Preserve current foreign generated-test hunks.
- Shared seam: `core/codegraph-defuse-extractor-contract/src/lib.rs`,
  `qv2/quanta-adapters-dfg/src/multi_lang_dfg_adapter/build_pipeline/fact_extraction.rs`
  and actual request construction/error/output-retainer callers discovered there.
- Existing authority interfaces: Kernel `cfg_parsed_tree/syntax_authority_v1.rs`,
  `language_plugin_port/{output_admission,temporary_backing_v1}.rs`, shared
  `current_read_v1.rs`. These are existing co-owned paths. Name one symbol/hunk
  steward for required edits and integrate the existing owner's changes first.
- Shared module registration, Cargo manifests/lockfiles and feature-policy changes
  go through I/A once, using their owning tools. B/C do not patch shared registries.
- Deliverable: real issuer→borrowed admission→physical producer→final keeper with
  exact source/backend authority and preserved ordinary compatibility. DFG's
  erased Native control has no source-operation/memory lease projection; the
  parser Java issuer is not a DFG facts issuer. Reuse canonical output/temporary
  interfaces and carry the actual original source control from outer ingress.
  The Java generated-field candidate is a partial precursor: CFG/facts creation
  exists but publication still refuses. Preserve that owner's implementation and
  finish its actual keeper/caller path rather than adding a second candidate.
  If the issuance handoff is absent, include that upstream bridge and all callers
  in S0 before controlled-success acceptance. No diagnostic grant, downcast of
  unavailable authority or post-allocation wrapping can authorize original custody.

### B — SDK tests then scalar snapshots

- W1: `qv2/quanta-runtime/tests/abstract_interpret_fluent_api_e2e_test.rs`
  has dirty authored timeout-zero and explicit live-deadline regressions for
  all three domains; execution/owner acceptance is pending. Production behavior
  is integrated; change it only if the regression reaches a reproducible defect.
- W2: `core/codegraph-lang-common/src/{c,go,typescript}_syntax_snapshot_v1.rs`,
  C/Go/TypeScript extractor scalar calculations and their affected owner tests.
- Integrated `syntax_snapshot_budget_v1.rs` (`f2d85a9`, QBC2/2) shares the 128 MiB
  local cap/overflow logic, but grants no original authority and
  charges no post-snapshot work. Shared ledger/helper mechanics belong to one
  birth/admission contract; grammar
  fields, binding/order and language interpretations remain distinct. Register any
  justified common module via A. Do not introduce a generic language visitor or
  an extra output issuer. Coordinate existing common/TypeScript writer claims.
- Deliverable: admitted storage before growth, actual post-snapshot charged work,
  original current/first-refusal checks before publication, temporary/final grant
  retention and refunds, and a remove/share/retain record for all three ledgers.

### C — Kotlin then caller/policy evidence

- W1/W2: `core/codegraph-defuse-extractor-kotlin/src/extractor/{mod,checked_node_v1}.rs`,
  affected facts/output helpers and specifically owned tests. No public plugin trait
  extension without an actual controlled-success caller.
- Dirty Kotlin source now checks retained backend and exact source before fact
  extraction; its authored same-length foreign-source/backend test covers the
  ordinary retained case. Native current, first refusal and output custody
  remain unqualified.
- W3: read/reconcile current-target planner, dominance, concurrency engine/schema,
  navigation, distinct `codegraph-import-graph`, `codegraph-core-cfg` call-graph
  SCC, rule execution, metrics and runtime community-cluster owner evidence.
  Request/oracle changes require real discovery; integrated source is not new work.
- Deliverable: Native same-length foreign source/backend/missing authority refusal,
  traversal/cancel/work/first cause and output lifetime; current/no-tree guarded with
  existing test-support context, without inventing a production caller.

### I — serial integration and proof publication

- Stage only complete reviewed bundles, inspect current foreign index/dirty hunks,
  `git diff --cached --check`, commit locally and record exact postimage/HEAD.
  Use the downstream normal coordinator/hooks for Semantica checkpoints.
- Producer→consumer signature changes, contract semantics and feature manifests
  ship in one compatible checkpoint. No partial cherry-pick or retired getter restore.
- GQLang documentation/contract changes stay in this repository; Semantica runtime
  source stays downstream. Push is a separate publication step.

## Local proof sequence and reuse

1. Resolve actual owning features/target/selector on the committed source using
   `scripts/quanta-build-cli owner resolve`; execute its admitted `owner run`.
   Do not copy historical feature classes or use raw Cargo as accepted owner proof.
2. Use the smallest affected owner oracle for each independent CP1/CP2; CP3 must
   prove the full changed producer/consumer closure. Require exact expected output
   and discriminating rejection, physical birth/lifetime and first-refusal witnesses.
3. Execute actual callback chronology, original5, affected language/custody owners;
   keep original8/JSON13/quarantine discovery and witnesses distinct. Reuse accepted
   source-equivalent inputs; changed manifests/dependencies require re-resolution.
4. Repair old45 into core/facade/PTA requests, then reconcile eleven SCC consumers
   before fixing total cardinality. One concurrency-engine owner covers its two
   consumers; no duplicate solver/Shape/PTA/taint requests. Tarjan10 alone omits
   additional callers. Include current Shape/Cost shared allocation feature closure.
5. Run affected mandatory feature/topology/Honesty/GOD and source policies. Earlier
   RED is historical until current reproduction. Repair attributable causes;
   baseline reapproval, new allows or reduced scope cannot discharge acceptance.
6. Re-freeze final dependency/source tuple, build actual SDK/runtime product and
   execute F2/F7 plus Graph/CHECK process result/cleanup obligations. Reuse this
   product proof for SUBSTRATE-PRODUCT instead of a second daemon queue.
7. Obtain release-owned installed trust and execute FCP-INSTALL positive CHECK.
   Missing external prerequisites remain explicit; local refusal proof is separate.
8. QGLang behavior changes require its smallest affected owner and one `just ci`
   for repository completion; `just qualify-local` only on clean committed HEAD.
   Prose-only plan publication supplies no new CI/runtime qualification.

Every accepted result records source/dependency pins, toolchain/features/root,
command, selected/executed/pass/fail/ignored/skip/leak counts, literal witnesses and
immutable normal owner receipts. Pending publication or zero selected tests fails
acceptance. Failed runs retain their source/receipt; fix only the reached owner,
then rerun the affected gate on a successor tuple.

## Exit rule

All committed retired carrier calls are gone; duplicate resource mechanics have
an explicit disposition; actual source/backend binding and original resource
lifetime survive the entire product path. Every required local owner/policy and
real product result has accepted matching evidence. External installed trust
retains its own state until its DoD is met; QG-DIAG is already resolved.
Move completed decisions to ADR/contracts and retire the packet only after its
own acceptance. Proposed query/optimizer development is not included in this exit.
