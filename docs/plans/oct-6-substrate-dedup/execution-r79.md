# Local parallel closeout execution — R79 onward

Updated: 2026-10-08. This is the executed successor of the [R78 dispatch](parallel-dispatch.md),
within the existing [18-ticket queue](tickets.md). All execution is local.
Source integration, WIP implementation, compilation and product acceptance are
separate states. The [R77 RCA](rca.md) retains its original inputs.

## Inputs and custody

- QGLang documentation input: `main` at `5bf60a1ad1e2415bb36865c0cde0308faffe749c`.
- Semantica initially `f574fa82bf123c817e80fc3afc795b4dd033ea3f`; the owned parser
  repair is now `d29ac508b4dec9d1a48dd359541a064fa43f1053` on local `main`.
- Concurrent coowner integration advances Semantica to `6bab127740a` (runtime
  search-evidence change) and QGLang to `753a2054f637` (nine-document checkpoint).
  The parser repair remains an ancestor; subsequent proof captures bind their
  own HEAD/overlay/sibling tuple. Neither external commit is this queue's test result.
- Other Semantica changes remain separately claimed shared WIP. Their source
  presence is not a main commit or current qualified result. Compatible reads
  below record particular files; only the leased snapshots bind execution inputs.
- Normal shared-main claims precede edits. A private candidate index contains
  only owned parser hunks; the normal coordinator serializes their main commit.
  Foreign hierarchy postimages are byte-preserved after integration.
- Canonical QBC capture immediately acquires the snapshot execution lease in the
  same process and holds it through the owner sequence. Earlier unleased roots
  were collected before later use; that operational gap is corrected. The lease
  ends after the run, so retained receipts/logs are evidence, not reusable roots.
- R84 rejects QGLang's changing document overlay before compilation. R85 uses
  the canonical topology owner's default committed-HEAD policy for this sibling
  (`753a2054f637`), after checking its dirty paths contain documents only; other
  required sibling code overlays and Semantica WIP remain captured. Exact
  materialization validation and the execution lease are retained. This selects
  a different declared input; it does not qualify the excluded document WIP.
- Evidence root: `/Users/songmin/.codex/task-evidence/oct8-closeout-execution-r79/`.
  The normal lane is `gqlang-closeout-r79-snapshot-artifact`, one heavy owner at a
  time in this execution queue. Source drift rejects a mixed input before tests.

## Actual parallel work and integration

| Lane | Executed change or result | State |
| --- | --- | --- |
| A / parser | Explicit hierarchy paths for `correspondence_v3`, `edges_v3`, `implicit_object_v3`; no-Java imports scoped to their existing test/Java features | Integrated `d29ac508b4d`; parser owner acceptance pending |
| A / native API | Core IR Foundation imports `codegraph_native_allocation_core::copy_string_v1` through its existing public root export | Authored WIP; do not integrate without the native supplier/manifests/caller packet |
| A / CFG reader | Explicit index and error signatures for the existing `IntervalReaderV3` callbacks; no algorithm or paid-storage change | Authored after reached R81 `E0282`; leaf format/diff PASS, successor owner pending |
| A / paid row type | Public opaque nominal `PaidBoolRowsV3<T>` for the sealed storage trait's associated type | Authored after R82 `E0446`; private fields and domain-only constructors/loan methods retained; leaf format/diff PASS |
| A+B / facts compile | Canonical FileId error import, two value-loan lifetimes and receiver checkpoint's typed unpublished-refusal mapping | Authored after R83; leaf formatting/whitespace checks PASS; general producer/output proof still open |
| A+B / core DFG compile | Existing sibling read import, common internal reader visibility and canonical `UnsupportedOperation` refusal | Authored after R85; companion Source/Dynamic rejection test corrected with zero-birth and cancellation assertions preserved; successor owner proof required |
| B / Original | Readback of authenticated member handoff, admitted relation/index/prepared rows and remaining final refusals | Existing coowner progress verified statically; no duplicate producer or fabricated grant added |
| C / consumer | Nine exact SCC/F2/F7 owner requests resolved; SDK2/community2 and eleven SCC scopes reconciled | `RESOLVED_ONLY`; no test acceptance inferred |
| C / lock | Root and nested lock add the existing DFG package to SDK executor's dev-dependency list | Exact one-line delta each; full locked metadata PASS; coupled manifest WIP not separately committed |
| I / execution | Frozen F2 request runs; reached causes routed to their owners; parser source candidate committed normally | Finished RED receipts retain zero tests and their exact input |

The integrated parser candidate contains two files and no foreign hierarchy
hunks. Normal staged patch integrity, exact-index rustfmt and changed-line safety
hooks pass. Hook-deferred semgrep and full compiler/product acceptance are not
claimed. See `owned-parser-candidate.json`, `owned-parser-integration.json` and
`owned-parser-postimages.json`.

Native allocation's current physical owner is the untracked
`packages/core/codegraph-native-allocation-core/` (20 files). Its workspace
member/tier and lock record, Core IR Foundation
(`packages/core/ir/foundation/`) dependency/reexport, tree-sitter
dependency/reexport and Kernel dependency/caller migration are not in committed
`d29ac508`. The live tree-sitter removal of its old `native_box_birth_v3.rs`
also belongs to that single-owner migration. An alias-only commit would omit
the supplier and preserve a second allocation implementation in committed code.
Coordinate exact compatible postimages; broad root manifest/lock staging would
include unrelated coowner changes. This execution reuses the existing coowner's
native-core implementation.

## Reached owner results

F2 minimal request `c674329c3f09d8c1f7ca80b9d0893c45` uses
`quanta-cli-ingress-runtime`, lib test, module selector
`dispatch::operation_binding_registry_v1::native_mutation_binding_tests`, policy
`feature-isolation:quanta-cli-ingress-runtime.no-default.01d5e8939f20`.
Its required witnesses are
`every_declared_native_mutation_has_one_matching_runtime_binding` and
`admitted_rename_preview_obeys_mutation_provider_availability`.

| Attempt / source digest | Actual result | Root cause / successor |
| --- | --- | --- |
| R79 / `0e4feb375030c8d2a6ea5d6418359ef5e562679cb33a1c6a358d97a79b56f9e3` | Finished RED, exit101, zero tests | Foundation private native module import (`E0603`); public root alias repair authored |
| R80 / `a3eb835ec14f077bd7c61ab06277f4dc84caa7d3b2c53df5e9982427d5bcaf24` | Finished RED, exit101, zero tests | SDK executor gained DFG dev dependency after an earlier lock repair; exact two lock edges repaired |
| R81 / `ae7fd46950fa02702a8169de55a9d08de2335b404e1d83cb48f4430862f5ee6f` | Finished RED, exit101, zero tests | Passed those prerequisites; CFG span reader's `.get(index)` reaches `E0282` at `cfg_span_index.rs:100`; owning type-inference repair authored |
| R82 / `d64ab3ed0ec449dc5a1421bd8b810350a5de97f94f17a8ce9600a3591447a863` | Finished RED, exit101, zero tests | Passed the reader repair; public span storage mode leaks restricted `PaidBoolRowsV3<T>` (`E0446`); opaque nominal type visibility repair authored |
| R83 / `f7b6f8d91a73780e7a8ef5574d34c28a74555bbb9816ed7609fe88d9325e15fc` | Finished RED, exit101, zero tests | Passed CFG prerequisites; facts crate reaches one canonical import, two borrowed-projection lifetimes and one typed checkpoint-error conversion; A/B repairs authored in disjoint owning hunks |
| R84 / no issued owner input | Source capture rejects, exit1; compiler not started, zero tests | External QGLang document overlay changed after capture; `source-capture-r84-failure.json` records the rejection; successor chooses committed frontend input |
| R85 / `68024eb04f3f7413dbf08f92b7d15d7a5bb3db3f9c4076f91262962664366a7c` | Finished RED, exit101, zero tests | Committed frontend capture succeeds; facts compiles. Core DFG reaches internal read import/visibility and a nonexistent native refusal variant; three repairs authored, companion test expectation follows the canonical enum |

Each run has its own `f2-native-owner-rNN.json`, `owned-proof-results-rNN.json`
and archived stdout/stderr. No required witness ran. These are compile failures,
not registry, Preview or deadline behavior failures. Dependent parser checks are
not launched after a failed first owner. The parser no-Java request is
`f3e26abbbf692f6d7ce8b84d44a5d9e2`; all-features is
`d3e8b5e5b2506be77ba2660c4b36afc4`.

### Lock resolution proof

`c-r80-lock-proof.json` records exactly one added `quanta-adapters-dfg` dependency
row in each existing SDK executor package. All other package records and registry
versions/sources/checksums are preserved. Full
`cargo metadata --offline --locked --format-version 1` passes root
(447 members / 798 resolved packages) and nested (327 / 751), with non-null
resolve graphs. These resolved counts differ from total lock package records.
The root lock postimage is `622c2e7e9a3bade55c4165d6a5f2b37d56ac0ee8d7e80a7cdbec366bec22d248`;
nested is `333b6a66f51634a438acde73ae836f8ce4d096d520c5f2a4d768ba5f6d293a7d`.
R77's no-deps metadata result did not establish this full resolution.

## Current Original source boundary

The [proof handoff](../../work/substrate-repair-checkpoints-2026-10-07.md)
keeps independent historical and coowner source observations separate.
The B-lane fresh readback at `d29ac508` distinguishes these authored parts:

- Parser `original_inputs_v3.rs` lends path/source/Tree C/semantic projections/
  authenticated FileId/output from the same member. DFG
  `original_workspace_evidence_issue_v3.rs` consumes that loan and validates
  FileId/path. The old external `members_v3` signature mismatch is superseded.
- PTA `OriginalNativeRelationLoweringOutputsV3` retains its operation/group and
  reuses the relation classifier/emitter. `DfgLoweringIndexV1::from_dfg_original_v3`
  has the 33-category pre-growth census. Prepared member rows and a void-only
  constraint transaction exist. These source cuts have no acceptance in this run.

The remaining success implementation is concrete:

1. General Java callable facts must join actual CFG G / Parser B / FileText S
   loans and output A, pay rows before growth and retain the opaque facts keeper.
   The private empty writer and public generated getter constructor do not supply
   this general API; the normal context/extractor still builds ordinary facts.
2. DFG `issue_original_callable_evidence_v3` still refuses generated and ordinary
   callable publication. Retained source-world/digest-map and Tree C publication
   also refuse. Complete these owning births under the same operation/output.
3. PTA refuses nonempty relation outputs and nonempty skip-definition/allocation
   ownership maps; current transaction coverage is void return only, without
   general uses/callsites/occurrence joins. Complete admitted phases and the final
   retained transaction before removing their typed refusals.

The existing `original_facts_keeper_retains_paid_rows_until_last_drop_v3` fixture
covers generated getter rows, foreign-root rejection, cancellation and refund.
It does not prove the general callable, DFG source-world/Tree C or PTA transaction.
Wrapping already allocated ordinary results or dropping rows cannot close these
boundaries. Preserve first typed refusal and publish no partial output.

## Remaining execution and completion criteria

1. **FCP-COMPILE / P0:** repair only newly reached owning compile causes; capture
   the complete supplier/caller tuple, pass parser no-Java/Java and actual
   runtime/SDK owners, then integrate the compatible source packet normally.
2. **CHECKED-RETAINED-CONTROL / P0:** finish the three success boundaries above;
   prove same-operation loans, admission before growth, first refusal, cancellation
   and final-drop refund through actual producers.
3. **Independent owner proof / P1:** execute Shape/Cost, PTA/taint/DFG, the three
   pending SCC scopes, SDK2/community2 and F2/F7 after each complete closure is
   available. The C request map (`c-consumer-owner-request-map-r79.json`) is not
   a result. `c-scc-continuity-d29-r83.json` verifies the eight earlier receipts'
   publication hashes: dominance3 and worklist77 match pinned committed `d29`,
   with no external path dependencies. Rule34, concurrency engine38/domain6,
   metrics14, import graph32 and core CFG30 have changed IR-build dependency
   closure and require successor owner proof. This continuity does not qualify
   moving WIP or later `6bab`. Existing accepted core/facade and scoped
   D4/D5 are not reassigned as source repairs.
4. **Policies and product:** run affected mandatory gates and receipt-bound real
   Graph/CHECK process oracles. Positive installed CHECK requires release-owned
   signed x86_64 Linux profile/immutable slots/independent approval/key/generation;
   this Darwin host has no installed positive result. FCP-INSTALL does not block
   independent local compile/algorithm proof. Reuse evidence for product rollup.

No ticket closes from static readback, request resolution, a failed compile or
mixed-source counts. Hosted CI, push and installed release publication are not
performed. Full closeout remains incomplete.
