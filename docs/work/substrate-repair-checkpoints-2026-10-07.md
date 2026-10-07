# Substrate repair checkpoints — 2026-10-07

Owner: Semantica runtime/SDK/API. QGLang owns frontend compilation and its
host-agnostic contract; the downstream repairs below introduce no new engine,
compiler, external library or hosted CI.

This supplement reconciles work after the `d0e303223c67` source observation in
`docs/work/remaining.md`. Main integration, private source changes,
owner proof and product qualification are separate states. Unfinished D1–D7
consumer/policy/product acceptance remains in that index.

## Integrated repairs

All SHAs below are local Semantica main ancestors. No remote publication is
claimed.

| Checkpoint | Before → after | Focused evidence |
| --- | --- | --- |
| `fdda8a5e997` — F6 | Client factory/cache publication could race terminal close → open check, factory and cache insertion share the lifetime lock; close invokes callbacks outside it; failed factories publish nothing. | Complete affected Python owner: 12 passed. Original close-first behavior fails the independent regression. |
| `5d3f87f47ab` — query production | Generic query owners referenced a feature-gated GQLang JSON control and lacked production imports/dependencies → existing JSON control moved byte-for-byte to neutral source-issuance owner; old GQLang path reexports it; scoped dependencies/imports/rollback bounds repaired. | Frozen QBC production check passed. This is compilation evidence, not PTA or product execution. |
| `457868f1c4f` — F10 | Agent requests could use a different root and ignored file/budget restrictions → startup captures one canonical directory; request admission rejects root mismatch, authored file lists and unknown budgets before SDK work. | Affected API/lifetime suite: 37 passed. Original wrong-root HTTP success fails the regression. |
| `0c3c26c90bc` — F5 | Public apply-changes discarded degradation, API emitted `ok=True`, projection lost native codes → SDK preserves typed outcome plus code/detail reasons; API reports degraded `ok=False` with applied data. | 43 affected cases passed, no skips. Three independent original-behavior regressions fail as expected. A broader unchanged suite exposed real Rename producer drift plus transport fixture drift. Rename request drift is repaired below; that earlier broad suite is not claimed green. |

| `975d1a3a41a` — F7 engine | Caller timeout ignored → one absolute deadline across existing SCC/worklist/narrowing and final result publication; lowering keeps typed stop and adapter covers all three domains. | Whole owning libraries: engine 73, lowering 58, adapter 21 passed; no filters or skips. |
| `e46cf07f5b1` — F2 Python | Current closed contract requires cap on every phase; SDK omitted it and Apply exposed no caller limits → Preview/Rollback retain null; Apply takes explicit typed positive u64 output/live limits and rejects absent/invalid limits before Preview. | 321 cases across the SDK execute owner and new cap/phase contract tests passed, no skips. Original Preview producer fails generated contract admission. |

| `8edd1b0733c` — F4/F9 Q3 | Cap was checked after neighbor insertion; composite slices retained BFS discovery direction and could accept bounded prefixes → admit before insertion, typed Incomplete, original induced eligible PDG edges. Empty-PDG failure precedence is retained. | Fresh frozen `a620b157998`: Q3 module 37/37 passed. Original cap oracle executes 1 and fails with root incorrectly admitted at cap0. Runtime projection acceptance remains separate. |

| `ecf12903bf3` — F7 native wire | Deadline stop lost its finite error/category contract → append payload-free RuntimeError stop, preserve old wire tags, map to AnalysisTimeout/ResourceLimit and close minimal serde feature. | Wire integration 2/2 and timeout/category projection 1/1 passed on their frozen owner sources. Concurrent main PTA feature ownership change is preserved; whole latest-main qualification is not claimed. SDK caller conversion remains a separate candidate. |

### Code owners

Paths are relative to `semantica-codegraph-v2/`.

- F6/F10: `apps/api/api/dependencies.py`, `main.py`, `routes/agent.py`.
- F5: `packages/sdk/python/codegraph_sdk/client.py`,
  `_product_apply_changes_projection_v1.py`, `apps/api/api/routes/indexing.py`.
- Query production:
  `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/source_issuance_control_v3/json_projection_v3.rs`
  and `quanta-runtime-query-executor-owner/src/query_executor/` consumers.
  The calculation is retained; moving its module does not add a second owner.

## Remaining repair candidates

Native and runtime Slice candidate source changes are on the private branch
`codex/oct7-substrate-query-owner-build-r1`; they are not yet claimed integrated.

| ID | Structural change | Acceptance still required |
| --- | --- | --- |
| F2 native | Add the missing canonical `mutation.refactor-rename` runtime binding. Retain existing typed backend, provider checks and Preview/Apply controls. Exact mutation inventory is the oracle, not a refreshed numeric count. | Execute native binding parity and feature-off refusal; prove public Preview reaches existing dispatch/backend; preserve phase/cap/host refusals. |
| F4/F9 | Admit each new eligible PDG node before insertion; distinguish zero cap, exact fit and omitted work. Composite bounded prefixes return typed Incomplete. Derive Chop/Dice PDG edges from final membership in original orientation. Runtime derives CFG blocks and occurrence metadata from final membership and retains original CFG edge kinds/counts. Included parameter provenance remains available while authored-statement display excludes parameter-only rows. | Q3 owner: 37 passed, including 8 new regressions. Q3 is integrated at `8edd1b0733c`; original traversal failed the independent cap oracle. Execute runtime output/statement and SDK refusal oracles on final source. |
| F7 native | Solver/lowering/adapter are integrated. Append a payload-free RuntimeError deadline refusal and preserve it through native SDK plus exhaustive timeout/category mappings; do not invent QueryId. | Engine owner: 73 passed; lowering owner: 58 passed. Whole adapter owner: 21 passed. Native wire integration 2/2 and timeout/category projection 1/1 passed on their recorded frozen sources; SDK projection proof remains pending. Check default30s, iteration distinction, all domains, zero/past deadline and non-timeout rejection controls. |

F7 does not promise hard interruption inside user callbacks, collection sorting
or lowering/projection calls. The deadline is checked at cooperative work
boundaries and after those calls; an overdue result is never published as success.

The candidate native wire stop appends `AbstractInterpretDeadlineExceeded` as
variant 39. Existing variants 0–38 keep their identities and shapes. Older readers
reject the new variant; frontend ExecutionDoc schema/digests are unaffected.

### Rename Apply support boundary

`crates/quanta-cli-ingress-runtime/src/dispatch/mutation.rs::execute_standard_mutation_v1`
currently returns `mutation_refactor_rename_apply_response_owner_unavailable_v1`
before SDK effects for Apply: no production terminal/replay response issuer is
installed. A caller cap is necessary but does not create that owner. Preview,
Rollback and feature-off native binding acceptance are separate checks.

Public explicit caller limits, shared by `refactor()` and `code_agent()`:

```python
from codegraph_sdk.refactor import MutationMaterializationCapV1

request.execute(materialization_cap_v1=MutationMaterializationCapV1(
    max_live_bytes_v1=4096, max_output_bytes_v1=1024,
))
```

Existing zero-argument `execute()` now rejects before Preview instead of sending
an unrepresentable Apply request. Preview/WorkspaceEdit signatures are unchanged.
The example demonstrates request authority, not production Apply success.

## Proof and failure discipline

- Frozen QBC owner source `0f36a0bae935`: Q3 module 37/37 passed. Original query source `4f5ec1a1509` executes one independent zero-cap fanout test and fails: returned `{root}` instead of `{}`. The first Q3
  attempt compiled zero tests because an existing test read private `file_id`;
  its accessor repair is included in the private candidate.
- Frozen QBC owner source `de5b7683ae40`: whole engine 73/73 and whole lowering
  58/58 plus whole adapter 21/21 passed. These receipts do not prove later native SDK wire edits.
- Rename attempt on `0f36a0bae935`: zero tests; foundation failed on three
  unused memo trait imports. Those imports are already removed on newer main.
  A fresh private candidate imports committed main rather than modifying a
  stale snapshot or weakening warnings.
- Earlier PTA run on `f8b73032d50`: zero executed tests, 13 adapter compile
  errors. That is a historical blocker, not an asserted error count for current
  main. Recheck its successor source before remediation.
- Record source SHA/digest, committed sibling pins, recipe/features, actual
  selected/executed/pass/skip counts and immutable original receipts. Never
  compose receipt counts into a whole-current qualification claim.

Local custody directory:
`~/.codex/task-evidence/oct6-substrate-dedup/recovery-20261006/query-owner-build-r1/`.
Original logs and source specifications are archived there; they are local
receipts, not repository-distributed or installed-product evidence.

### Successor proof inputs

- Native wire `f7addd36e49`: 2/2 tests passed with the canonical minimal
  `input-cell-surface,wire-base-identity` class. The feature's missing
  `serde/derive` edge was repaired at its owner; no new dependency was added.
- Timeout/category mapping `b8644ab5c6b`: 1/1 passed with 33 nonselected cases.
- Q3 successor `a620b157998`: source digest
  `ffbc26a108204a4585a2c5ca3814cc893d434756cd120cf4bfe75e10a5b2aad8`.
  It adopts committed main DFG import repair `c26a43639b6`; the older Q3
  `b8644ab5c6b` compile failure is superseded, not a current source defect.
- The older R4 runtime jobs were cancelled after that dependency was superseded;
  pending work is rerun on R6. Cancellation supplies no test verdict.
- Rename provider proof now additionally enters the actual admitted context,
  registry and worker before the existing backend. Its source is `7eea70e4a06`;
  an isolated handler receipt alone does not close that path.

## Final acceptance order

1. Complete independent owner regressions and preserve typed failures.
2. Import only reviewed, owned paths into local main with normal hooks and
   shared-main coordination; recheck drift immediately before import.
3. Retire only source-confirmed integrated repairs from the remaining-work index.
4. Execute affected mandatory consumer/policy gates on one frozen main tuple.
5. Execute actual native daemon/product acceptance. Installed Linux CHECK still
   needs release-owned signed immutable slots and independent approval; helper
   tests and compiler success do not supply those prerequisites.

## Frontend local CI

`just ci` completed on QGLang HEAD `68ff02ef443f` with its then-current
working overlay. The runner recorded source snapshot
`f3a565f81f5d412a2aefd011319c6db0946b35f4706f786e3b321cff7b485dd6`:
all groups passed; nextest 1514/1514 with zero skips, doctest 10,
external-consumer positive 1 and negative compile-fail 23.
Subsequent edits here and to the remaining-work/ADR reconciliation are prose
updates; the receipt is retained at its original source. This is local CI,
not a clean committed-HEAD qualification receipt.
