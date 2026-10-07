# ADR-0005: Substrate reuse and consumer projection ownership

- Status: Accepted
- Date: 2026-10-07
- Scope: Oct-6 D1–D7/F1/D6 and the subsequent integrated consumer/owner repairs
- Implementation owner: `semantica-codegraph-v2`; this ADR changes no runtime source

## Context

The substrate audit and its execution packet mixed architectural choices,
completed code checkpoints, failed candidates, test requests and product proof.
The retained implementation choices reuse existing owners. Code integration and
qualification must remain distinct, especially while downstream main moves.

## Decision and integrated checkpoints

The source checkpoint column records the retired packet's integration lineage,
not qualification of today's downstream checkout.

| ID | Durable decision | Recorded main checkpoint |
| --- | --- | --- |
| D1 | Standalone Shape is the calculation owner. Factories and concrete imports/exports use it; Cost retains Cost responsibilities. Shape-only and PTA-derived feature closure require their own consumer proof. | `712b5e8c` |
| D2 | `codegraph-graph-scc` owns the borrowed iterative SCC traversal; graph-utils reexports compatibility paths. Generic, PTA, taint and later consumers share the kernel; domain projection, output order/IDs and typed Compute/Stop remain explicit. Admit work/frame/output allocation before growth. | Initial graph-utils core `df2a867ad70`; adapters `712b5e8c`; owner move `ff8f2ce935f` |
| D3 | Keep active runtime MemSSA pack lowering; remove dormant DFG build-input entry and exclusive helpers/exports. Preserve PortDFG and shared digest/policy/finalizer/sort. | `d88f0a2ed7a` |
| D4 | SCV2 contribution calls existing `rrf_contribution_v1`. Preserve K/clamp, accumulation order, identity/ties, top-k and duplicate refusal. Separate fusion contracts retain their own owners. | `d88f0a2ed7a` |
| D5 | Keep base UnionFind, active solvers and retained pair sessions. Remove dormant Andersen UnionFind module/export after consumer census; do not add a compatibility wrapper or new trait. | `d88f0a2ed7a`; self-contained proof/topology `6dd27ad5f05` |
| F1/D6 | Native row variants, true identity, optional location and primary/supplemental summary are canonical. SDK projection precedes shared app field conversion. API, HTTP agent and public MCP preserve their own request/failure/envelope policies. | `c1f821db572` |
| D7 | Registry captures one bounded, validated immutable manifest source. Metadata/list/schema/dispatch project that same capture; direct init and retry preflight precede adapter/SDK initialization. Reject canonical-name and alias duplicates. | `b258f5919a5` |
| F6 | API client acquisition checks open state, creates the repo client and publishes its cache entry under the same lifetime lock as close. Close marks terminal state and clears the cache under that lock, then invokes the closer outside it. Failed factories publish no cache entry. | `fdda8a5e997` |

Do not centralize a broken contract. An absent symbol location stays absent;
file-candidate presentation does not fabricate a symbol ID. Primary and
supplemental counts retain their separate meanings, so total is not forcibly
raised to the number of all emitted rows. Deep immutable private manifest
records still project ordinary dict/list structures for existing consumers.

Neither similar names nor similar algorithms establish interchangeable contracts.
Dependency task planning, relation optimization, abstract interpretation,
runtime state instances and provider generation ownership remain distinct.
No new universal crate, engine, solver, SMT dependency or hot-reload service was
selected by this work.

## 2026-10-07 source reconciliation

Rechecked committed Semantica
`d0e303223c67449e58ea5217b65169a42e0a07b0` through read-only Git captures,
separately from its live dirty overlay. The paths below are relative to that
repository; `qv2/` abbreviates `packages/analysis/quanta-v2/crates/`.
Use the fixed commit when reproducing this observation.

| Decision | Retained implementation authority |
| --- | --- |
| D1 | `qv2/quanta-adapters-shape/src/shape_analyzer_adapter.rs`; runtime-surface `src/analysis/mod.rs` factory/export use standalone Shape, while Heap/Cost remain separate |
| D2 | `packages/core/codegraph-graph-scc/src/scc.rs` owns calculation; `qv2/quanta-graph-utils-v1/src/lib.rs` reexports. PTA/runtime-analysis/taint and later domain consumers preserve their own projections |
| D3 | `qv2/quanta-adapters-dfg/src/lib.rs` retains PortDFG-facing MemSSA exports; the retired build-input entry is absent |
| D4/D5 | Retrieval-kernel `src/scv2_exact_fusion.rs` calls `rrf_contribution_v1`; `packages/core/pta/engine/src/solver/mod.rs` retains base UnionFind and has no retired Andersen UnionFind module |
| F1/D6 | SDK `_product_search_projection_v1.py` precedes shared `current_search_projection.py`, consumed by API search and MCP QueryAdapter |
| D7 | `apps/mcp/mcp/domain/query_surface_registry.py` bounded capture/validation/deep freeze; `apps/mcp/mcp/main.py` preflights before constructing adapter/SDK |
| F6 | `apps/api/api/dependencies.py::ProductClientLifetimeV1`; existing `apps/api/tests/test_product_client_di_lifecycle.py::test_lifetime_*` |

F6's seven existing lifetime tests executed unchanged against the captured owner
classes and exception, with only their pure component dependencies loaded:
selected/executed/passed `7/7/7`, skipped `0`. They cover close-first,
acquire-first, close-once, typed unavailable, per-repo reuse, factory failure
and closer execution outside the acquisition lock. This is isolated component
proof, not execution of the complete HTTP test module or native sessions.
The remaining Rust/feature/policy/product scopes still require their own
source-bound acceptance; an implementation census does not close them.

## Subsequent integrated repair decisions

The following choices are present in committed Semantica
`577a225c3459c27f566643ea2c9bcb95cac71cca`. Their twenty-three recorded integration
checkpoints are ancestors of that source. This is a source reconciliation;
the focused results below retain their original frozen inputs and are not
new results on that main commit.

| Owner / IDs | Retained decision | Integration lineage |
| --- | --- | --- |
| API lifetime/root, F6/F10 | Serialize acquisition/publication with close. Capture the startup canonical directory and reject request-root mismatch, authored file restrictions and unknown budgets before SDK calls. | `fdda8a5e997`, `457868f1c4f` |
| Mutation result, F5 | Preserve typed outcome and native code/detail through SDK and API. Degraded application retains applied data with `ok=False`; it does not imply fresh analysis. | `0c3c26c90bc` |
| Rename request/binding, F2 | Preview/Rollback encode the required null cap. Apply requires caller-issued positive u64 live/output limits before Preview. Register the exact canonical native operation and retain provider/phase/issuer refusals. | Python `e46cf07f5b1`; native source `33ff8c41b3b` |
| Abstract interpretation, F7 | One absolute cooperative deadline spans lowering, SCC/worklist/narrowing and final publication. Preserve its typed error through native wire/category and SDK conversion. No hard interruption inside callbacks is promised. | `975d1a3a41a`, `ecf12903bf3`, SDK source `33ff8c41b3b` |
| Q3 and Runtime Slice, F4/F9 | Admit before node insertion; exact-fit completion is distinct from capped partial output. Chop/Dice use induced eligible original PDG edges. Runtime retains original CFG direction/kinds and final statement-occurrence membership; bounded incomplete output cannot publish complete success. | `8edd1b0733c`, prerequisite `0dd691a651c`, runtime `1a5cb6b0d29` |
| Shared source/feature owners | Keep JSON issuance control at its neutral owner with the original path reexport. Retain one shared Java/Python generation-result owner, exact borrows and caller-aligned cfg predicates. Feature closure does not enable Java to satisfy Python minimum. | `5d3f87f47ab`, `7bbdcc1d4e3`, `4518fd6fec5` |
| Receipt/event/lifecycle | Private decoder seeds stay crate-visible; public wire/generic decoding is unchanged. Event witnesses observe Visible before canonical lease commit and Committed after it. Expose the existing borrowed lifecycle across its owner boundary without inventing a second lifecycle. | `375173d94dc`, `00a54727bae`, `8fe0dc8ca3a` |
| Original-control reason custody | A reason's failed retained birth releases an independent owned group issued by the same original control, while a live fingerprint retains its own group. Preserve marker/group identity, detached last-clone custody, replay and allocation-refusal controls. | Six-file source absorbed at `33ff8c41b3b` |
| Dependency and lint closure | Reconcile existing root/nested lock edges without new package/version changes. Preserve public unsafe obligations while narrowing the allowance to the two exact-pinned custody methods and test module. Remove the serializer's redundant import without changing its tuple/preimage or warning policy. | `669a4608343`, `931b2ed9db8`, `400db0750c2`, `e73d3274f80` |
| Cross-source build seeds | Full locked/offline Cargo metadata includes every resolved local package/patch (`source == null`). Sanitize its existing build artifacts before publishing an imported seed; missing donor paths and metadata/clean failures remain typed failures. Registry artifacts retain existing reuse. | `b453a31eb847` |
| Retained TARGET path | Compare borrowed native path bytes with the canonical UTF-8 member after checked work addition and original-control admission. Preserve exact phase-specific refusal; no lossy alias, component normalization or unadmitted String allocation. Test imports point to their existing owners without changing parser mappings. | `933daac0014`, `b7fe68c10aa` |

The native deadline stop appends `AbstractInterpretDeadlineExceeded` at tag 39;
old tags 0–38 retain their identities and shapes. Older readers reject the new
tag. This downstream wire change does not alter frontend ExecutionDoc digests.
Production Rename Apply still refuses before SDK effects when its terminal/replay
response issuer is absent. A caller cap and installed host ceiling do not issue
that authority.

### Focused evidence and outstanding boundaries

Later checked-consumer checkpoints are integrated on Semantica main:

- `35f1c81946e`: Java fixtures use their checked-node and fallible identity owners.
- `a8778714805`: the diagnostic bundle facade invokes the existing canonical
  Java surfaces once through callback-scoped syntax reads; empty-source and
  malformed-health authority remain explicit refusals.
- `6580317bb84`: CFG adapters reuse the existing text-access reexport and bind
  checked-node return lifetimes; stale Go import, Java extra reference and
  TypeScript name shadowing are removed. All language features compiled in the
  recorded R39 Native graph, which then failed at Java DefUse before any cases.
- `26f16598905`: Parse/HIR share the wire owner’s admitted native-path byte
  comparison; CFG preserves current refusal before syntax custody. The existing
  wire dependency moves into production and three fixtures use checked APIs.
  R44 owners passed 14/5/413/13 on this committed checkpoint without private
  cursor files. Three cursor files still await the five original-issued
  memory/work/cancellation controls; product acceptance remains separate.

Whole-Java 139 and common traversal 7 passed on the fixed R38 candidate with
zero ignored cases. Its three cursor paths are now integrated as `dc2611b96e1`,
with exact candidate postimages confirmed. The five original-issued
memory/work/cancellation controls still did not execute because DefUse failed
to compile; these historical passes do not discharge that acceptance. DefUse's removed raw getter is a
separate checked-node consumer migration, not justification to restore raw
syntax projection or reparse an original-controlled input.

Historical R45 supplier inventory at `e7bae76f70b0` separates five plugin owners (Rust/Go/Kotlin/C/TypeScript,
ten retired `CfgParsedTree` getters) and seven DefUse owners (seven preparsed
carrier getters). T1’s canonical Rust owner reaches Rust/Go E0599, exit 101 and
zero tests; the other entries are static source findings. Shared cursor/source
integration and focused R38/R44 results do not qualify these consumer graphs.
Existing `ParseResult` getters are distinct from the retired carrier API.

The historical successor read-only audit through committed Semantica `ff8f2ce935f` confirmed C
plugin (`801fbd6777a`), C DefUse (`480b5fb445d`), Kotlin DefUse (`736a8047245`)
and Go DefUse (`070f4f96770`) source integration. Remaining carrier calls are
eight plugin and four DefUse entrypoints at that source; WIP and claims remain separate
from committed behavior. D1–D7 source checkpoints are ancestors of this source
and retain their integrated decisions. The later `ff8f2ce935f` cut moves SCC
calculation into `codegraph-graph-scc`, leaves graph-utils as compatibility
reexports, and migrates PTA adapter/Andersen consumers. The old SCC file was
renamed, not retained as dead duplicate source. Its owner/caller verification
requires current selectors rather than the retired graph-utils `scc` request.

The final ticket audit through committed Semantica `b32361e9d83` additionally confirms
Kotlin plugin (`ecc0c5784b4`), Rust/Go plugins and Rust DefUse (`ed94af892c0`),
Java DefUse (`e14eeda8604`), TypeScript plugin/DefUse (`b32361e9d83`) and the coupled
callback chronology repair (`aac43629734`) are integrated. The sole remaining
retired carrier call is Python DefUse. Checked-read migration does not alone
prove exact binding, original output admission or runtime acceptance.
The current-target planner, dominance and concurrency engine/schema deadlocks
also reuse the canonical SCC. Tarjan10 includes PTA adapter/Andersen, but its
selectors and historical45 do not cover all callers. The
[current tickets](../plans/oct-6-substrate-dedup/tickets.md#d2-caller-coverage)
retain the expanded caller proof without assigning new SCC implementations.

C/Go/TypeScript grammar snapshots are distinct scalar projections. Their three duplicate local
cap/overflow mechanics and original-control/output acceptance remain in
[CHECKED-RETAINED-CONTROL](../work/remaining.md#substrate-verification).
The five SDK mapper tests are distinct from engine/adapter deadline behavior and
missing real SDK deadline result coverage. These unfinished tasks are scheduled
in the [successor execution plan](../plans/oct-6-substrate-dedup/readme.md);
this audit is source evidence, not a new owner or product execution receipt.

The checkpoint record reports separate original-source results:

| Scope | Recorded result |
| --- | --- |
| Python lifetime / root / mutation / Rename producer | 12 / 37 / 43 / 321 passed on their respective owner sources |
| Deadline engine / lowering / adapter | Whole owners 73 / 58 / 21 passed |
| Deadline wire / category | 2 / 1 passed; category target had 33 nonselected cases |
| Slice Q3 / prerequisite CFG / Runtime helper | 37 / 47 / 6 passed; four independent runtime output/membership regressions executed |
| Minimal receipt / event / JSON integration / quarantine | 69 / 173 / 13 / 130 passed; JSON selected one complete integration module with 24 other target cases filtered |
| Seed sanitizer / TARGET-path and existing parse owners | Recorded sanitizer 13 passed; R30 path 5, parser mapping 4, parse 15 and file identity 6 passed, zero ignored. These keep their original sources. |

These counts are distinct scopes, not a sum or a same-source product verdict.
The original preissuer allocation failure was reached before its repair; its
RED receipt is retained. Metadata resolution and scoped policy checks are not
Rust test execution. Historical zero-case build failures remain historical.

Native Rename and SDK deadline source integration do not close their execution
acceptance. Revalidation after memory-owner changes, D1–D7 owner45, mandatory
policies, native process and installed acceptance remain in the
[remaining-work index](../work/remaining.md). The
[active proof handoff](../work/substrate-repair-checkpoints-2026-10-07.md)
retains current frozen candidate inputs, exact targets/witnesses and immutable receipt paths
while another task is executing that work. Retire that execution record after
its remaining acceptance is reconciled; do not use it as an API definition.

## Evidence boundary

The retired ledger recorded core SCC20, MemSSA pack16, SCV2 leaf34 and isolated
UnionFind225+2 canonical passes on different frozen sources. D7 recorded 70
Python owner/consumer cases. F1/D6 recorded SDK211 with 2 skips, consumers141
with 1 skip, installed wheel1 and Granian/scripted daemon1. Those figures are
historical component evidence, not one same-source product receipt.

The `dfc48c5` tuple's source6/relation5/formal9/JSON16 results, a different
tuple's budget39, generated topology and scoped GOD retirement cannot be
combined into current-main owner45 or product qualification. The PTA-first
attempt at that source failed compilation and executed zero tests. Later source
repairs do not turn that failed run into a pass.

Generated topology was regenerated and integrated. Scoped GOD retirement
reduced the selected entries from 14 to 7 with no outside changes. Global GOD
and Honesty ratchets remained RED; baseline approval, allow additions or scope
reduction are not substitutes for resolving those reports.

## Remaining work and recovery

Implementation work orders are retired. Unfinished Shape/PTA/taint consumer
proof, DFG/taint acceptance, generated-policy closure and native product proof
remain in the [remaining-work index](../work/remaining.md#substrate-verification).
The subsequent integrated repairs above retire their code work, with Native
F2/F7 execution acceptance still separate. F3 pagination and F8 startup leak
were excluded for lack of a demonstrated supported-path defect.

Original write sets, selectors, frozen tuples, archives and JSON receipts remain
recoverable from Git at the [consolidation baseline](0001-frontend-campaign-consolidation.md).
They are historical inputs and must be revalidated before another owner run.
