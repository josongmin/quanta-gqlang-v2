# Substrate closeout tickets

Updated: 2026-10-07. Audit inputs: QGLang `01cfe39ca0571b4fc099d41b24628fda70e754fe`,
Semantica `871224b2059422a8f36eca91ffb0c9119238ac9c`. Committed source was read;
foreign dirty overlays were excluded from that audit. R49 authored source and
execution attempts are recorded separately in the plan; they are not passing
Rust or product results. All execution is local.

[Remaining work](../../work/remaining.md) owns current states. This index adds
scope, priority, owner and DoD for its **19 active closeout IDs**, plus the
resolved QG-DIAG decision. Proposed
A00–A15, optional Index and optimizer/incremental batches remain in their
[separate roadmap](../../work/remaining.md#proposed-code-query-work); they are
not defects or prerequisites for these tickets. Paths below are relative to
Semantica; `core/` means `packages/core/`, `qv2/` means
`packages/analysis/quanta-v2/crates/`.

## Queue and state snapshot

| ID | Priority | State | Remaining kind / owner |
| --- | --- | --- | --- |
| CHECKED-LANG-SUPPLIERS | P1 | PROOF_PENDING | All inventoried plugin cuts integrated; owner acceptance |
| CHECKED-DEFUSE | P0 | PROOF_PENDING | All inventoried source cuts integrated, including Python27d950; actual owner proof remains |
| CHECKED-RETAINED-CONTROL | P0 | OPEN | Shared output admission, C/Go/TypeScript controls and Kotlin binding; existing downstream owners |
| F7 | P1 | ACCEPTANCE_OPEN | Real SDK deadline regression plus existing owner proof |
| QG-DIAG | P2 | RESOLVED | Existing private exact-field consumer contract; ADR-0008 |
| CHECKED-CALLBACK-FIRST-CAUSE | P1 | PROOF_PENDING | Integrated bridge/Kernel repair; common + parser owners |
| CHECKED-CURSOR | P1 | PROOF_PENDING | Original-issued traversal five; parser owner |
| D1-PROOF | P1 | INCOMPLETE | Shape factory/SDK/features; adapter + consumer owners |
| D2-PTA-PROOF | P1 | INCOMPLETE | Core/facade/PTA owner and request repair |
| D2-TAINT-PROOF | P1 | INCOMPLETE | Taint domain/public contract and 50k depth owner |
| D2-CALLER-COVERAGE | P1 | PROOF_PENDING | Eleven additional SCC consumer scopes; added by this audit |
| D3-PROOF | P1 | INCOMPLETE | PortDFG MemSSA and taint promotion |
| D4/D5-CUSTODY | P1 | UNVERIFIED | Rebind historical SCV2/UnionFind accepted inputs |
| SUBSTRATE-POLICY | P1 | INCOMPLETE | Affected feature/topology/Honesty/GOD policies |
| F2 | P1 | PROOF_PENDING | Native registry and real Preview path |
| FCP-COMPILE | P1 | UNVERIFIED | Current committed SDK/runtime product build |
| FCP-PTA | P1 | REVERIFY | Historical PTA bootstrap refusal; reproduce on current source |
| FCP-PROCESS | P1 | NOT_RUN | Real Graph/CHECK daemon process results |
| FCP-INSTALL | P1 | EXTERNAL_PREREQUISITE | Release-owned installed Linux CHECK trust |
| SUBSTRATE-PRODUCT | P1 | UNVERIFIED | F1/D6/D7 acceptance rollup; reuse FCP product proof |

P0 reflects a confirmed source migration/custody seam. A pending proof row does
not assert a current runtime defect. Old compile failures and policy RED remain
historical until reproduced. Request47 is a partial map repair, not the final
number after SCC caller reconciliation.

## CHECKED-LANG-SUPPLIERS

- Alias: `P-TS` now proof-only. All five inventoried plugin cuts are integrated;
  TypeScript joined C/Rust/Go/Kotlin as `b32361e9d83` during this audit.
- Source: TypeScript `snapshot_from_cfg_tree_v1` performs the retained checked read
  and feeds the shared scalar snapshot to HIR/dispatch/type/identity calculations.
  Its remaining `ParseResult` getter at684 is a separate live API.
- DoD: exact bundle/HIR/quarantine/ID/order/spans and ordinary parity plus typed
  controlled-carrier refusal pass at the actual owners. The public trait has no
  current parameter. No TypeScript or other integrated migration is assigned again.
  Shared scalar output/control obligations belong to CHECKED-RETAINED-CONTROL.
  The `codegraph-import-graph` consumer repair `c82e3566f99` must preserve the
  exact operation/refusal detail through its actual error projection.

## CHECKED-DEFUSE

- Alias: integrated Python cut; `D-TS` is proof-only. Python is committed
  `27d9505015e`; TypeScript is integrated `b32361e9d83` with the shared snapshot.
- Source: zero production retired-carrier calls remain across the inventoried
  owners. Python enters checked extraction through `extract_facts_with_context_v1`
  and `frontdoor_phases_v1`; its145 package files and manifest match the R49 snapshot.
- Change: accept the integrated extraction/binding/assignment/helper graph with
  owner tests; preserve exact CFG/function binding, fact order, assignments and typed failure.
  A raw shim or reparsing an original-controlled retained input fails acceptance.
- Dependency: shared output signatures must be coordinated with
  CHECKED-RETAINED-CONTROL. No replacement Python or TypeScript migration writer.
- DoD: the committed Python call is gone; independent expected facts plus
  mismatch/resource/first-refusal tests pass. Integrated C/Go/Kotlin/Rust/Java
  and TypeScript owner acceptance remains required, without duplicate implementation.

## CHECKED-RETAINED-CONTROL

- Aliases: `S0`, `CONTROL-SNAPSHOTS` (former `CONTROL-CGO`), `CONTROL-KOTLIN`. Coordinate with the existing
  downstream D0 facts/output and S0 final-output work; these are shared handoff
  boundaries, not extra writers or replacement Native allocation issuers.
- **S0 issuer prerequisite:** the DFG controlled ingress carries only erased
  `NativeAllocationControlV1`; ordinary file/source-only callers pass None.
  No canonical output-admission construction was found in its committed pipeline.
  The trait exposes reserve/work/grant validation, without source-operation lease
  projection or downcast. The generic original output admission needs actual
  `ServingSourceIssuanceControlV1` and a validated operation/memory lease.
  Its parser Java wrapper does not establish DFG facts issuance. Trace the outer
  retained ingress and final keeper before choosing the signature. If DFG lacks
  the handoff, include the upstream source/output bridge in this coupled scope;
  unavailable required authority refuses, ordinary compatibility stays explicit.
- **S0 output bridge:** `qv2/quanta-adapters-dfg/src/multi_lang_dfg_adapter/build_pipeline/fact_extraction.rs:14`
  and `core/codegraph-defuse-extractor-contract/src/lib.rs:492` carry tree/source/current
  but no output-admission interface. Carry the existing canonical admission through
  adapter→context→actual extractor and final facts retainer. Admit before physical
  growth; retain permits through the last consumer and refund on final drop.
- **C/Go/TypeScript controls and dedup:** `core/codegraph-lang-common/src/{c,go,typescript}_syntax_snapshot_v1.rs`
  had identical 128 MiB cap/checked-add ledgers. R49 extracts those mechanics
  into private `syntax_snapshot_budget_v1.rs`, preserving all charge sites;
  both boundary tests passed under QBC; checkpoint `f2d85a9` is integrated. C extractor `mod.rs:221`, Go
  `traversal/fact_helpers.rs:147` and TypeScript `mod.rs:1068,1100` calculate facts
  after the checked snapshot callback, without original current in that calculation.
  The common counter removes the three duplicated accounting mechanics; original
  admission remains a coupled cut. Retain distinct grammar projections.
  A local cap does not prove original temporary/output custody.
- **Kotlin binding:** `core/codegraph-defuse-extractor-kotlin/src/extractor/mod.rs:1338`
  R49 authors a checked-callback source/backend guard and same-length foreign
  source/backend regression. Exact-source output and ordinary unsealed
  compatibility passed in exact1/whole70 owner tests; checkpoint `df315e6` is
  integrated. Original current/output lifetime acceptance remains open.
- DoD: same-length foreign source/backend and missing required Native authority
  refuse before calculation; original low-budget/cancel/work/first-refusal cases
  fail before publication; temporary and output grants survive producer drop and
  refund at their actual last owner. Duplicate ledgers have a remove/share/retain
  disposition. Couple changed signatures and consumers in one compiling bundle.
- Current/no-tree can be tested with existing test-support context. This audit
  found no production caller for that state; no new production API is assigned.

## F7

- Alias: `SDK-DEADLINE`. Owner: `quanta-runtime` SDK tests. Production absolute
  cooperative deadline and typed mapping are integrated.
- Source: `qv2/quanta-runtime/tests/abstract_interpret_fluent_api_e2e_test.rs`
  now includes R49 real `execute()` timeout0 and literal normal-result tests
  for Sign/Interval/Constant; their owner execution is pending.
  Builder default is 30_000; existing mapper five assert a different boundary.
- Authored scope to qualify: real fluent `execute()` cases for timeout0 and
  literal expected results across sign/interval/constant, with exact typed error.
  Retain distinct iteration failure and deterministic engine/adapter
  final-publication checks at their existing behavioral owners. No sleeping oracle.
- DoD: SDK integration target plus engine/lowering/adapter/wire and mapper owners
  have separate accepted results; revalidate affected original custody/JSON/
  quarantine. Deadline begins at adapter `analyze`; do not claim whole-request
  or hard callback interruption. See [F7 test plan](test-plan.md#f7-separate-mapper-proof-and-actual-deadline-behavior).

## QG-DIAG

- State: RESOLVED as a contract decision, with no executable change.
- Actual parity/NDJSON consumers require the exact primary-only seven-field
  diagnostic projection. No registered client requires secondary locations or
  structured authority identity. Protocol `authority_error` remains a failure.
- The [language reference](../../language/README.md#cli-and-diagnostics-stdio) documents
  the current private wire; [ADR-0008](../../adr/0008-private-diagnostics-primary-location-contract.md)
  owns the durable decision. Existing binary/NDJSON/parity tests own execution
  proof; this source readback does not fabricate a current test receipt.

## CHECKED-CALLBACK-FIRST-CAUSE

- Alias: `S0-CALLBACK`. Source and fixtures integrated `aac43629734`.
- Owners: common typed/ordinary bridge, infra controlled syntax and Kernel
  metadata boundary; original controlled fixtures belong to `quanta-adapters-parser`.
- Focused evidence: R54 common callback2 and Kernel finisher5 passed. Parser
  chronology still requires its actual original-control owner result; retain
  `PROOF_PENDING` until that consumer scope passes on the accepted input.
- DoD: actual callback error keeps identity and causes zero owner/Kernel post-polls;
  recorded original refusal retains priority; success still postvalidates custody.
  Execute the existing chronology fixtures. A finisher-only or fake poll test
  cannot replace original-control acceptance. No source repair is assigned anew.

## CHECKED-CURSOR

- Source cursor3 is integrated `dc2611b96e1`; reviewed R43 consumer repairs are
  also absorbed. R38/R44 proofs retain their historical tuples.
- DoD: execute the [five literal original-issued controls](test-plan.md#original-issuer-traversal--exactly-five-existing-controls)
  after actual compile closure. Require no backing growth, work/cancel and pruning
  first-refusal witnesses; five executed/pass, zero ignored. No reimport assignment.

## D1-PROOF

- Standalone Shape factory/export is integrated. Owners: Shape adapter, runtime
  SDK, session/composition and query converters.
- DoD: pure Shape, PTA-derived Shape, disabled and Cost-only combinations pass
  exact expected output and typed absent-port refusal; both existing Shape gates
  and the three feature-tree diagnostics have matching current inputs.
- Latest `07093dbe72a` shares `PointsToResult::referenced_allocations_v1` through
  `quanta-contract-pta-result-shared`. Include actual Shape/Cost allocation
  projection oracles and the new no-default feature classes `399e8a289332`
  (without Java metadata) / `3d4e77a3e18b` (with metadata) in current resolution.
  Their policy definitions alone are not execution proof.

R64 accepted Shape projection20 and Cost heap projection20 on one independent
frozen source, with exact orphan/endpoint and truncation outputs. This does not
execute the runtime/SDK/gate matrix or both shared-allocation feature closures.

## D2-PTA-PROOF

- Core is `codegraph-graph-scc`; graph-utils is compatibility reexports. Owner
  requests must replace the old zero-case graph-utils `scc` selector with core
  `scc` plus facade `facade`, and add actual PTA adapter `scc_collapsing::tests`.
- DoD: core/facade, PTA adapter, Andersen and runtime precise-query results retain
  membership/order/IDs, graph closure, preallocation limits and original typed stop.
  Reuse the existing PTA engine solver request for its SCC governance witnesses.
  Tarjan10 source includes these owners; its execution remains separate.

## D2-TAINT-PROOF

- Owners: taint solver and public contract tests. Use the integrated shared SCC.
- DoD: integration/differential/property/wire/public contract oracles pass with
  successor-only domain and unchanged order/IDs/resource/stop semantics. Explicitly
  select ignored `deep_chain_50k` as its own request; zero selected is failure.

## D2-CALLER-COVERAGE

The historical45 map and Tarjan10 filters do not select these eleven consumer
scopes. This is a proof-inventory gap; source reuse is integrated. Resolve current
owner targets/features and match existing accepted receipts before adding runs.

| Owning package | Actual source boundary / required oracle |
| --- | --- |
| `quanta-runtime-analysis` | `queries/interprocedural/incremental/current_target_scc_planner_v1`: partition/order, malformed graph vs resource refusal, payload sharing/final refund |
| `codegraph-graph-dominance` | `src/lib.rs` no-exit sinks: minimum representatives/order, one-byte-short refusal, deep iterative cycle |
| `codegraph-core-concurrency-engine` | `deadlock/tarjan_runner.rs` and `deadlock_algo.rs`: real witness edges/order, reentrant/self-cycle policy, unregistered neighbor and resource refusal; one package proof covers both consumers |
| `codegraph-core-concurrency` | `domain/deadlock/wfg_graph.rs`: emission/member order and cycle identity, deep cycle, one-byte-short and malformed key refusal |
| `quanta-adapters-navigation` | `name_resolver/import_graph_snapshot.rs`: import SCC membership/order and malformed/resource behavior |
| `quanta-runtime-execution` | `gqlang/rule_exec_v1`: real rule partition/strata/result semantics and typed SCC stop |
| `quanta-runtime-metrics-wire` | `max_call_depth_compute_v1.rs`: computed depth across SCCs, recursion/unregistered graph and resource refusal |
| `quanta-runtime` | `retrieval/port_impls/index_projection_writer/scv2_orchestration/generation_index/community_cluster.rs`: exact membership/order/identity and empty/cyclic input |
| `codegraph-import-graph` | `src/substrate.rs` and `src/substrate/cycle_work_tests.rs`, integrated `daf8bb00bfa`: bounded cycle partition, actual edge witness/lexical choice, missing return/malformed graph and typed resource refusal before partition publication. Separate from navigation |
| `codegraph-core-cfg` | `cfg/schema/src/domain/call_graph/record.rs::scc_tests` and call-graph stats; canonical SCC reuse `6a795b466f2` preserves historical component order. Independent exact graph, 12k chain, one-byte-short and inconsistent reverse-index oracles; taint Zipper consumes the same schema surface and needs its real criticality result proof |
| `codegraph-core-abstract-engine` | `abstract/engine/src/solver/worklist.rs::scc_tests`: independent all3-node cyclic-edge oracle, deep cycle, one-byte-short workspace and unregistered successor; actual solve deadline/widening/determinism consumers. Integrated `90e6bb0ea3d` intentionally adds canonical resource bounds and typed `RejectedGraph`; source presence is not accepted execution proof |

R63 accepted rule34 and R65 accepted dominance3, concurrency engine38/domain6,
metrics14 and core-CFG call graph30 on one issued input. Current-target/navigation
and community keep their compile prerequisites. R66 explicitly adds the actual
taint-Zipper consumer result request and source-bound import graph request; no
new SCC implementation or feature policy is assigned. Both are accepted32/32
on the same issued source, including the actual Zipper criticality/selector and
malformed-index outputs. Blocked callers and final aggregate proof stay open.

DoD: each affected owner has accepted matching evidence or a resolved request
with actual required witnesses. Freeze the complete request count afterwards.
Do not rerun Shape/PTA/taint work twice or mistake the facade type test for caller
semantics. Extend policy/owner inventory only where current coverage is absent.

## D3-PROOF

- Active PortDFG MemSSA lowering is retained; dormant DFG build-input entry removed.
- DoD: actual DFG adapter/pack/taint-promotion owners pass canonical memory output
  and closure/failure oracles. Pack16 has historical proof; old compile RED means
  its blocked request executed zero cases, not a current defect reproduction.

## D4/D5-CUSTODY

- SCV2 uses common RRF; base UnionFind/active solvers retained, dormant Andersen
  UnionFind removed. Historical SCV2 leaf34 and isolated225+API-absence2 are accepted
  on their original sources. R64 additionally accepted solver225/225 and the
  separate API-absence2/2 on the independent R63 frozen candidate; SCV2 and final
  aggregate custody remain open.
- DoD: attest unchanged owner/dependency inputs or rerun changed owners for the
  final tuple. No second RRF implementation, new UnionFind trait or redundant run.

## SUBSTRATE-POLICY

- DoD: affected feature/dependency/topology, test-honesty, GOD/Honesty, FFI/input
  and generated-source gates pass on the final tuple. Earlier RED needs current
  reproduction/attribution before a code repair is assigned. Do not reapprove a
  baseline, add an allow or narrow scope to obtain green. Local reports retain
  exact source and normal publication status; pending is not an accepted verdict.

## F2

- Native registry/mutation source is integrated. Owner: `quanta-cli-ingress-runtime`.
- DoD: minimal/full registry and admitted context→worker→real Preview backend
  cover canonical operation parity, six mutation IDs and provider-off refusal.
  Preserve PreviewPatch and missing Apply terminal/replay issuer refusals.
  Use [existing selectors and witnesses](test-plan.md#f2-native-registry-and-preview).
  This ticket does not authorize an Apply-success implementation.

## FCP-COMPILE

- Current reached closure is recorded in the
  [R53–R60 handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#r53r60-reached-repairs-and-successor-input).
  R52 semantic contract55 passed, but later SourceWire pre-test errors and
  unexecuted SDK/product consumers prevent whole product acceptance. R63 rule
  execution is accepted34/34 and integrated `a23ced26e09`; community stopped
  before tests at56 committed parser errors. Clean
  main's all-features Java path references missing Foundation
  `OriginalSemanticSharedBackingV3`; its dirty implementation/export must be
  integrated as the complete owning bundle, not hidden in a test overlay.
- DoD: freeze committed Semantica/GQLang/Index and actual original-session/QMP
  closure, resolved toolchain/features/root; canonical SDK/runtime product build
  succeeds. Record current diagnostics if it fails and open only the reached owner
  repair. Historical31/5 compile counts do not describe this source.

## FCP-PTA

- Historical seed/bootstrap lifetime refusal is a re-verification input.
- DoD: current owner/bootstrap succeeds, or an actually reached
  `PtaOccurrenceAuthorityRefused` identifies its producing owner and precise remaining
  repair. New commits alone do not close it; old error prose is not a new bug.

## FCP-PROCESS

- Dependency: FCP-COMPILE and actual artifact/host capability. Installed positive
  CHECK also requires FCP-INSTALL; local helpers/Preview do not.
- DoD: [real Graph/CHECK process entries](test-plan.md#product-external-installation-and-frontend-completion)
  return exact positive/refusal output on one tuple, with selected/executed/pass/
  skip/leak counts and process cleanup. Linux Graph positive and non-Linux refusal
  are distinct obligations; signed CHECK controls preserve their actual host policy.

## FCP-INSTALL

- Owner: downstream release/installation trust. External prerequisite remains open.
- DoD: release-owned signed x86_64 Linux profile, immutable root-owned slots,
  independent approval/key and active generation, followed by actual installed
  positive CHECK proof. Generic aarch64 or scripted daemon output does not close it.

## SUBSTRATE-PRODUCT

- Aggregate acceptance for canonical Native SDK→app conversion (F1/D6) and one
  bounded immutable MCP manifest capture/preflight (D7).
- DoD: exact public identity/location/summary and each consumer's request/error
  policy pass; manifest duplicate/schema/preflight behavior passes; required native
  daemon/installed results reuse FCP-PROCESS/FCP-INSTALL on the same tuple. Existing
  Python proofs may be reused only with owner/dependency continuity. No second
  product execution queue is created for these aliases.

## Completed-source exclusions and downstream handoff

- D1–D7 implementations, integrated C/Rust/Go/Kotlin/TypeScript plugin and C/Rust/Go/Kotlin/Java/TypeScript
  DefUse migrations, cursor3 and callback chronology repairs remain integrated.
- Graph ingress uses one witness and Path=`NodeSequence` / Reachability=`Endpoints`;
  schema8 golden and producer contract trees/vector pin match this GQLang source;
  EXISTS true/true and false/false rejection cases and Rename corpus exist. Their
  current product proof is retained under the acceptance rows, not reopened source work.
- F3 token-loss, F8 startup-socket-leak and unsupported GraphRAG/detail-loss claims
  stay excluded; no demonstrated current public success-contract violation was found.
- Related downstream [DefUse ticket](../../../../semantica-codegraph-v2/docs/plans/oct-7-ir-structural-closeout/tickets/D0-DEFUSE-SCOPED-READ-CUT/TICKET.md),
  [facts/output ticket](../../../../semantica-codegraph-v2/docs/plans/oct-7-ir-structural-closeout/tickets/D0-NATIVE-FACTS-CUT/TICKET.md)
  and [final output ticket](../../../../semantica-codegraph-v2/docs/plans/oct-7-ir-structural-closeout/tickets/S0-FINAL-NATIVE-OUTPUT-CUT/TICKET.md)
  are broader foreign dirty work. This index records overlapping boundaries without
  replacing their owner's queue, accepting their WIP, or assigning duplicate writers.
  Refresh their source/claims/barriers at the next actual handoff.
