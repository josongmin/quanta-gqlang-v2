# Substrate closeout tickets

Updated: 2026-10-08. Initial audit inputs: QGLang `01cfe39ca0571b4fc099d41b24628fda70e754fe`,
Semantica `871224b2059422a8f36eca91ffb0c9119238ac9c`. Committed source was read;
foreign dirty overlays were excluded from that audit. R49 authored source and
execution attempts are recorded separately in the plan; they are not passing
Rust or product results. All execution is local.

[Remaining work](../../work/remaining.md) owns current states. This index adds
scope, priority, owner and DoD for its **18 active closeout IDs**, plus the
resolved QG-DIAG decision and accepted D4/D5 scope. Proposed
A00–A15, optional Index and optimizer/incremental batches remain in their
[separate roadmap](../../work/remaining.md#proposed-code-query-work); they are
not defects or prerequisites for these tickets. Paths below are relative to
Semantica; `core/` means `packages/core/`, `qv2/` means
`packages/analysis/quanta-v2/crates/`.

Latest repair, integration and owner results: [R79 onward execution](execution-r79.md).
R77 source/diagnostic RCA and execution order: [R77](rca.md). Historical
observations below keep their original inputs. Core20/facade1 are accepted on
the fixed f574 tuple; current WIP compilation is not accepted.
The [R78 dispatch matrix](parallel-dispatch.md#ticket-dispatch-matrix) assigns
each active ID to one accountable slot with explicit wave/checkpoint dependencies.
Its write boundaries and actual capacity constrain parallel execution; ticket
assignment alone does not transfer another owner's files.

## Queue and state snapshot

The original Shape/SCC dedup source and owner routing are committed on main;
no additional traversal implementation is assigned. Rows below retain their
acceptance/publication obligations and the subsequently added Native/Original
scope. Implementation, frozen owner proof and product qualification are separate.

| ID | Priority | State | Remaining kind / owner |
| --- | --- | --- | --- |
| CHECKED-LANG-SUPPLIERS | P1 | PARTIAL | Ordinary owners accepted and Rust regression integrated; controlled refusal/custody remain |
| CHECKED-DEFUSE | P0 | PROOF_PENDING | All inventoried source cuts integrated, including Python27d950; actual owner proof remains |
| CHECKED-RETAINED-CONTROL | P0 | OPEN | Genuine member handoff, admitted PTA/DFG publication and general facts output authority; local-ledger/Kotlin source repairs integrated |
| F7 | P1 | ACCEPTANCE_OPEN | Real SDK deadline regression plus existing owner proof |
| QG-DIAG | P2 | RESOLVED | Existing private exact-field consumer contract; ADR-0008 |
| CHECKED-CALLBACK-FIRST-CAUSE | P1 | PROOF_PENDING | Integrated bridge/Kernel repair; common + parser owners |
| CHECKED-CURSOR | P1 | PROOF_PENDING | Original-issued traversal five; parser owner |
| D1-PROOF | P1 | INCOMPLETE | Shape factory/SDK/features; adapter + consumer owners |
| D2-PTA-PROOF | P1 | INCOMPLETE | Core20/facade1 accepted; PTA/Andersen/runtime consumer proof remains |
| D2-TAINT-PROOF | P1 | INCOMPLETE | Taint domain/public contract and 50k depth owner |
| D2-CALLER-COVERAGE | P1 | PROOF_PENDING | R87 core CFG30/domain6/engine38 accepted; changed import graph32 reaccepted atR89; rule/metrics/current-target/navigation/community and final-input continuity remain |
| D3-PROOF | P1 | INCOMPLETE | PortDFG MemSSA and taint promotion |
| D4/D5-CUSTODY | P1 | SCOPED_ACCEPTED | SCV2 34 + solver225/API2 accepted; active source closures match main f574 |
| SUBSTRATE-POLICY | P1 | INCOMPLETE | Scoped Honesty/GOD/boundary checks pass; Python-search floor policy bundle integrated as3b8f86d; whole affected policy closure remains |
| F2 | P1 | PROOF_PENDING | Native registry and real Preview path |
| FCP-COMPILE | P0 | BLOCKED_COMPILE | Native packet remains WIP; R93 exact finite error regression1 is accepted on its own source. R97 parser captures four Kernel repairs and stops on14 errors, zero tests; later minimal-import postimages require a fresh capture. R92 F2's91 is historical. Complete current V1/V2/facts/taint caller/authority migration, broad compile and compatible integration. Separate R82 candidate: interproc1105 and actual fallback1 have committed owner receipts; R155 canonical-error capture is superseded, and R158 Taint CHECK now stops at thirteen Kernel diagnostics with zero tests. Live caller closure and coupled integration remain open. |
| FCP-PTA | P1 | REVERIFY | Historical PTA bootstrap refusal; reproduce on current source |
| FCP-PROCESS | P1 | NOT_RUN | Real Graph/CHECK daemon process results |
| FCP-INSTALL | P1 | EXTERNAL_PREREQUISITE | Release-owned installed Linux CHECK trust |
| SUBSTRATE-PRODUCT | P1 | UNVERIFIED | F1/D6/D7 acceptance rollup; reuse FCP product proof |

P0 reflects a confirmed source migration/custody seam or prerequisite compile block. A pending proof row does
not assert a current runtime defect. Old compile failures and policy RED remain
historical until reproduced. Request47 is a partial map repair, not the final
number after SCC caller reconciliation.

## R77 dependencies and assignment

Source capture is an execution prerequisite of FCP-COMPILE. It is not another
product defect or ticket. An independent owner can proceed when its own closure
is complete; blanket serialization behind unrelated Original work is unnecessary.

| Active ID | Remaining assignment | Prerequisite / completion boundary |
| --- | --- | --- |
| FCP-COMPILE | Finish compatible module/API/supplier/caller bundle | Exact tuple capture, no-Java/Java and actual SDK/runtime compile, coordinated integration |
| CHECKED-RETAINED-CONTROL | Finish real Original producer and final output | Complete owning supplier closure; members, admitted transactions/publication and general facts keeper |
| CHECKED-DEFUSE | Accept integrated language extraction | Affected context/output closure; exact facts/binding and rejection at actual extractors |
| CHECKED-LANG-SUPPLIERS | Accept controlled carrier and custody | Ordinary libraries already accepted; controlled source/output handoff required for remaining cases |
| CHECKED-CALLBACK-FIRST-CAUSE | Accept parser chronology | Callback repair integrated; affected parser compile and real bridge witnesses |
| CHECKED-CURSOR | Run original-issued five controls | Cursor source integrated; parser/Original issuer closure |
| D1-PROOF | Accept Shape/Cost SDK and feature matrix | Actual adapter/SDK features compile; existing Shape20/Cost20 scopes reused |
| D2-PTA-PROOF | Accept PTA adapter, Andersen and runtime-analysis | Core20/facade1 reused; actual PTA request blocked by fixed-tuple parser compile |
| D2-TAINT-PROOF | Accept taint contracts and ignored 50k depth | Actual solver closure and explicit ignored selector; zero cases fail |
| D2-CALLER-COVERAGE | Reconcile eleven additional consumer scopes | Reuse matching receipts; current-target/navigation/community and final input continuity remain |
| D3-PROOF | Accept PortDFG/MemSSA and real promotion | Actual DFG/taint owner closure; integrated Pack16 retains its own input |
| FCP-PTA | Reverify historical bootstrap refusal | Real bootstrap owner; a current reached refusal determines any new repair |
| F2 | Accept Native registry→worker→Preview/provider-off | Existing minimal/full feature closures; preserve Apply refusal |
| F7 | Accept actual SDK deadline/domain behavior | Frozen SDK compile closure; mapper5 and deadline2 are distinct; integrate authored tests after proof |
| SUBSTRATE-POLICY | Run final affected mandatory gates | Integrated tuple and exact policy scope; retain gate strength |
| FCP-PROCESS | Run actual Graph/CHECK process oracles | Receipt-bound binaries/host; installed positive CHECK also requires FCP-INSTALL |
| FCP-INSTALL | Supply and verify release-owned trust assets | Signed x86_64 Linux profile, immutable slots, independent approval/key and active generation |
| SUBSTRATE-PRODUCT | Roll up F1/D6/D7 and actual product evidence | Reuse the same FCP process/install results and qualified public projections |

The [RCA](rca.md#confirmed-causes) defines the cause evidence and source boundaries;
the [test plan](test-plan.md) defines executable witnesses. Eighteen IDs are not
eighteen unfinished implementations. No percentage is inferred from their count.

## CHECKED-LANG-SUPPLIERS

- Alias: `P-TS` now proof-only. All five inventoried plugin cuts are integrated;
  TypeScript joined C/Rust/Go/Kotlin as `b32361e9d83` during this audit.
- Source: TypeScript `snapshot_from_cfg_tree_v1` performs the retained checked read
  and feeds the shared scalar snapshot to HIR/dispatch/type/identity calculations.
  Its remaining `ParseResult` getter at684 is a separate live API.
- Execution: R67 whole libraries passed C57/Go54/Rust76/TypeScript120/Kotlin43,
  but Rust's retained-source witness was raw-string text and never ran. The
  one-file repair `f574fa82bf1` activates it; R70 actual Rust77/77 also verifies
  the committed IR dependency change. R71 accepts C57/Go54/TypeScript120/Kotlin43
  on that same source, whose active dependency closures match main. Ordinary
  libraries are accepted; controlled original-carrier refusal remains open.
  Use terminal receipts and actual witness names; a grep-based
  `#[test]` census does not establish executable discovery.
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
- **S0 output bridge:** `qv2/quanta-adapters-dfg/src/multi_lang_dfg_adapter/build_pipeline/fact_extraction.rs`
  and `core/codegraph-defuse-extractor-contract/src/lib.rs::DefUseFactsParseContextV1::single_parse` carry tree/source/current
  but no output-admission interface. Carry the existing canonical admission through
  adapter→context→actual extractor and final facts retainer. Admit before physical
  growth; retain permits through the last consumer and refund on final drop.
- **R74 caller observation, superseded handoff:** runtime-analysis→PTA→DFG was
  already reachable. Its proposed external `members_v3` argument is removed in
  favor of the genuine parser member loan at R96; current runtime and SDK
  fixture callers both pass `parser_handoff_v1`. No extra argument repair is
  assigned. Complete and prove the existing workspace formal/alias/catalog/extraction births;
  transaction issuance and retained DFG publication still intentionally refuse
  UnsupportedOperation. This narrows the earlier missing-caller claim without
  closing compile or runtime acceptance. [Code map](code-map.md#existing-java-producer-callers-and-remaining-refusal--r74)
  records the complete caller paths; no second issuer/keeper is assigned.
- **R96 current Original authority:** the R74 external `members_v3` selector is
  superseded by the parser's authenticated member and borrowed same-cohort
  handoff visitor. QRA's authored Original request issuer takes the actual
  `PinnedQuerySessionGenerationSealV1` and derives membership/provider-alias
  authority from its source closure. A narrow existing-owner custody validator
  checks the pin without serializing and discarding its identity material;
  source/batch census validation precedes destination births. This cut and its
  SDK regressions have no compiler/test acceptance yet.
- **R96 actual SDK boundary:** normal Java capture, receipt admission and the
  NativeLowering input still require ordinary inventory/concrete workspace
  handoff. The receipt frontdoor explicitly returns `UnsupportedAllocation` on
  Original. This is an existing unsupported feature, not a demonstrated runtime
  defect. Direct genuine QRA proof must not be reported as end-to-end SDK support;
  any future product cut must extend this canonical caller path rather than
  create a second attribution issuer. The older sealed Parser fixture has an
  ordinary generation pin and cannot prove Original pin custody. Evidence:
  `r96-original-sdk-caller-boundaries.json` at the existing recovery root.
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

- Core is `codegraph-graph-scc`; graph-utils is compatibility reexports. The
  old graph-utils `scc` selector selects zero cases. Resolve the actual PTA
  adapter `scc_collapsing::tests` next.
- Fixed-pair R75 accepts core whole-lib20/20 and facade whole-lib1/1 with
  verified publication on Semantica `f574fa82bf1`; locators and source digest are in
  [remaining work](../../work/remaining.md#substrate-verification). This does
  not cover PTA consumers.
- The PTA adapter `scc_collapsing::tests` owner request on the same input is
  RED before execution: parser compilation reports 105 errors and selects zero
  tests. Repair the coupled supplier/parser closure before repeating it.
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

R83 continuity verifies all eight earlier publication hashes against pinned
committed `d29`: dominance3 and worklist77 match their owner input. Rule34,
concurrency engine38/domain6, metrics14, import graph32 and core CFG30 have
changed conservative IR-build dependency closure and require successor proof;
this change does not assert a behavior regression or feature-specific execution.
The six exact recipes are resolved, not executed. Current-target, navigation and
community remain three separate pending scopes. See [execution record](execution-r79.md).

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
| `codegraph-core-abstract-engine` | `abstract/engine/src/solver/worklist.rs::scc_tests`: independent all3-node cyclic-edge oracle, deep cycle, one-byte-short workspace and unregistered successor; actual solve deadline/widening/determinism consumers. Integrated `90e6bb0ea3d`; R72 whole-lib77 observes all four witnesses with finished/hash-verified publication on its clean f574 sibling tuple |

R63 accepted rule34 and R65 accepted dominance3, concurrency engine38/domain6,
metrics14 and core-CFG call graph30 on one issued input. Current-target/navigation
and community keep their compile prerequisites. R66 explicitly adds the actual
taint-Zipper consumer result request and source-bound import graph request; no
new SCC implementation or feature policy is assigned. Both are accepted32/32
on the same issued source, including the actual Zipper criticality/selector and
malformed-index outputs. Blocked callers and final aggregate proof stay open.
R72's existing abstract worklist77 result is accepted for that owner scope;
its separate input is not added to the R70/R71/R73 same-source total. Reuse it
when source/recipe custody match; no duplicate worklist run is assigned.

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
  separate API-absence2/2 on the independent R63 frozen candidate. R71 SCV2
  fusion34/34 and R73 solver225/225 + API-absence2/2 are now accepted on one issued
  source. All three have finished GREEN, verified publication hashes and active
  dependency closures matching main `f574fa82bf1`; SCV2's external index pin also
  matches. State: **SCOPED_ACCEPTED**.
- DoD met: changed active IR-build dependency was projected from committed main
  and D5 rerun, with all227 prior executable witness names retained. No second
  RRF implementation, new UnionFind trait or redundant run. Future changed-input
  revalidation belongs to common policy gates; product aggregate stays in
  SUBSTRATE-PRODUCT/FCP rather than keeping this accepted scope indefinitely open.

## SUBSTRATE-POLICY

- DoD: affected feature/dependency/topology, test-honesty, GOD/Honesty, FFI/input
  and generated-source gates pass on the final tuple. Earlier RED needs current
  reproduction/attribution before a code repair is assigned. Do not reapprove a
  baseline, add an allow or narrow scope to obtain green. Local reports retain
  exact source and normal publication status; pending is not an accepted verdict.

## F2

- R79/R80 frozen minimal-owner attempts both stop before tests: R79 reaches
  Foundation's private native module import; R80 reaches a newly missing SDK
  dev-dependency lock edge. The owning repairs are authored and R81 re-executes
  the same request. These are compile prerequisites, not Preview behavior results.
- Native registry/mutation source is integrated. Owner: `quanta-cli-ingress-runtime`.
- DoD: minimal/full registry and admitted context→worker→real Preview backend
  cover canonical operation parity, six mutation IDs and provider-off refusal.
  Preserve PreviewPatch and missing Apply terminal/replay issuer refusals.
  Use [existing selectors and witnesses](test-plan.md#f2-native-registry-and-preview).
  This ticket does not authorize an Apply-success implementation.

## FCP-COMPILE

- **Scoped fixed-input acceptance:** the separate R82 recovery evidence family
  reaches R142 interproc whole lib/all-features **1,104/1,104 GREEN**, committed
  QBC publication, source `e33930b2c05cdc0825447ddf467a808b92631e8f731d6a001f46c38f6f81ff6b`.
  See the [current checkpoint](../../work/substrate-repair-checkpoints-2026-10-07.md#fixed-candidate-compile-and-ownership-checkpoints).
  The broad ticket stays `BLOCKED_COMPILE`: live external callers, canonical
  Kernel/SourceText/RuntimeCtx feature closure and coordinated main integration
  remain required. The R79 execution family below retains its own inputs.
- **Earlier R79-family reached cut:** R92 parser stops on two finite error projections
  whose existing owner has authored the successor; R93 exact regression1 is
  accepted with immutable publication on its own input. R96 parser passes that
  boundary and reaches18 Kernel errors before tests. Its guarded source and
  archived RED remain separate from authored successors. F2's R92 closure stops
  on91 interproc-contract diagnostics across22 files. Five hierarchy mounts and
  three private control signature/work-width repairs are authored with leaf
  checks, not accepted by a successor compile. See the exact [execution](execution-r79.md).
- **Coherent consumer migration:** use the canonical opaque result owner and
  existing ordinary/retained read contracts. Public heap/context-sensitive
  carriers cannot keep unconditional Clone/Eq over authenticated retained data.
  Propagate typed fallibility through V2 legacy/common projection, its17 caller
  files in PTA/runtime/runtime-analysis, the actual facts append path and taint
  access/call-resolution APIs. Missing current authority must never become
  empty facts, an empty call list or None. Include affected public trait/signature
  compatibility, independent ordinary result oracles and retained refusal tests.
  The existing producer claim does not cover this complete consumer cut. R96's
  source-only reconciliation finds active V1/V2 copy and facts/taint child owners;
  preserve their work and finish distributed callers rather than duplicating it.
  In-progress facts append must preserve storage/digest causes, atomic output
  and the original first error; alias/access/call consumers must distinguish
  missing rows from authority refusal. See the current partition in the execution.
- **Kernel successor:** finish source-authority feature mounts/imports and typed
  identity/observer error propagation; context/source-text/output contract consumers,
  callback generic error and feature-off empty storage must compile coherently.
  The owned envelope borrow, finite identity error forwarding and two observer
  issuance consumers have leaf checks and separate source postimages. R97 captures
  all four, no longer reports their diagnostics, and stops on14 other errors before
  tests. Three diagnostic files then change under the existing minimal-import
  owner;14 is not claimed as current live-main residual. These source repairs
  have no independent main commit of their co-owned files.
  Capture complete compatible postimages before parser no-Java/Java and actual
  SDK/runtime owners; keep R92 F2 and R96 parser counts bound to their own inputs.
- R79 integrates the three hierarchy mount corrections and feature-scoped imports
  in Semantica `d29ac508b4d`. The live Foundation native-copy alias now uses the
  existing public root export. Its native-core supplier/manifests and the two
  R80 lock rows remain coupled WIP. Full root/nested locked dependency resolution
  passed after the lock repair; Rust owner acceptance remains separate. See
  [current execution and integration boundary](execution-r79.md).
- R77 groups the fixed tuple's 105 parser diagnostics as 11 missing module mounts,
  32 unresolved/private names, four declaration/trait diagnostics and 58 denied
  unused imports. Parser and PTA SCC reach the same diagnostic set. Eight mounts
  have explicit corrected WIP paths; three hierarchy mounts still require review.
  The current alias metadata check succeeds with 447 workspace members, with
  unchanged manifest/lock hashes. This closes only that manifest prerequisite.
  See [cause and ownership](rca.md#1-the-committed-parser-packet-is-incomplete).
- R76 SDK attempt reaches a new dirty-source manifest prerequisite:
  Foundation's `semantic-original-admission` forwarded a query-floor feature
  using the package name instead of its existing `quanta_runtime_query_floor`
  dependency alias. The one-string owner repair preserves other WIP and passes
  offline locked all-features Cargo metadata (327 workspace packages), with
  unchanged manifest/lock during that diagnostic. It is not independently
  integrated. Subsequent whole-target and exact SDK attempts stop at source
  custody capture while sibling HEAD/overlays or Semantica files change; no SDK
  test execution or accepted owner publication is claimed. See the
  [R76 handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#r76-sdk-attempts-and-reached-manifest-repair).
- Earlier reached closure is recorded in the
  [R53–R60 handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#r53r60-reached-repairs-and-successor-input).
  R52 semantic contract55 passed, but later SourceWire pre-test errors and
  unexecuted SDK/product consumers prevent whole product acceptance. R63 rule
  execution is accepted34/34 and integrated `a23ced26e09`; community stopped
  before tests at56 committed parser errors. Clean
  main's all-features Java path references missing Foundation
  `OriginalSemanticSharedBackingV3`; its dirty implementation/export must be
  integrated as the complete owning bundle, not hidden in a test overlay.
- Current clean-main R72 parser all-features lib check reaches105 errors/exit101,
 0 tests on the fixed f574 sibling tuple. Its missing publication marker keeps
 it a pretest compile diagnostic. The [R72/R73 handoff](../../work/substrate-repair-checkpoints-2026-10-07.md#r72r73-d5-closeout-and-independent-main-diagnostics)
 separates that current block from independently published worklist and D4/D5
 acceptance. State: BLOCKED_COMPILE; existing module/supplier postimages remain
 co-owned, and full SDK/runtime build is unqualified.
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

## R169–R178 scoped Source/caller progress

FCP-COMPILE and CHECKED-RETAINED-CONTROL now include source-ready PTA
unit/status output slots, QRA's initial/request/result axes slots and the current
checked test-executor caller. The SDK genuine positive's retained-charge and
last-drop refund assertions are authored. They need complete consumer tuple
compilation and actual execution; no active ID is closed by source shape alone.

R174 nominal SourceWire9 is accepted with committed publication; R172's one
unused-import error belongs to the older fixed prefix capture, while live Source
uses the import in a separate snapshot-digest helper. The PTA early payload drop
review did not establish early reservation refund or a runtime safety defect.
Its comment was narrowed to the demonstrated lease/publication behavior.
Remaining owner/consumer/product oracles follow the
[proof order](test-plan.md#source-callback-and-caller-proof-order).
