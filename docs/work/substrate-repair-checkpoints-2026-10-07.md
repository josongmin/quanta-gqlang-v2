# Substrate repair checkpoints — 2026-10-07

Owner: Semantica runtime/SDK/API. QGLang owns frontend compilation and its
host-agnostic contract; the downstream repairs below introduce no new engine,
compiler, external library or hosted CI.

This supplement reconciles work after the `d0e303223c67` source observation in
`docs/work/remaining.md`. Main integration, private source changes,
owner proof and product qualification are separate states. Unfinished D1–D7
consumer/policy/product acceptance remains in that index.

## Current status

- The local main checkpoints/source packets are reconciled below. Event
  `00a54727baed`, lifecycle `8fe0dc8ca3a6`, and the nine source paths absorbed at
  `33ff8c41b3ba` remain integrated. This task added lock closure at
  `931b2ed9db820d4735732b8e1898837621e5f4e4` through the normal coordinator/hooks;
  its commit attestation is present. No push is claimed.
- R31 requalification freezes committed main
  `b7fe68c10aa2114e2ccba4ae78b56a39be4ccd76`, with
  committed gqlang `7ff7ed8290ed` and Index `8642fa9b3b47`. Its source digest is
  `4145303d4614a8a544cd68f6c88fb30c5356e6c8f9a01cfd930c562acee94859`.
  Moving foreign overlays are excluded; executions apply only to this tuple.
- Historical GREEN results remain event 173/173 and JSON 13/13 on R22,
  quarantine 130/130 on R24. Canonical memory-owner changes require affected
  JSON/quarantine execution on the new source; old receipts are not relabelled.
- The R24 SDK unsafe/forbid compilation conflict is historical: main `3ad1d3976d9`
  changed the crate to deny unsafe with a scoped output-custody override.
  Actual SDK execution on that replacement source is still pending.
- R25 Native minimal failed before tests because the committed lock lacked
  current manifest edges: zero executed cases. Root/nested lock closure now adds
  13/11 existing dependency edges, with no package/version/checksum changes.
  Both private-source `metadata --locked --offline` diagnostics passed; the
  lock checkpoint preserves both foreign lock deltas. This is no Rust verdict.
- R26 Native minimal reached dependency compilation and failed on an unused
  `serde::Serialize` import in the existing generation-row serializer: zero
  cases. Checkpoint `e73d3274f80` removes that import only; warning policy,
  serializer tuple/preimage and public signatures are unchanged.
- Checkpoint `400db0750c2` narrows the existing unsafe allowance from the whole
  custody module to its two documented methods plus test module. The existing
  exact override inventory now pins their containing source file. Both method
  signatures/bodies are byte-identical; no unsafe precondition is removed.
  Six affected boundary policies and the canonical scoped inventory passed.
- R27 Native and quarantine both stopped on four invalid PathBuf operations;
  R28 then exposed test-module path/import closure errors. All stopped before
  tests. R29 then stopped on a relocated vendored build script, also zero cases.
  R30 repairs the seed owner: path 5/5, parser mapping 4/4, parse 15/15 and
  file identity 6/6 passed, with zero ignored cases. The five paths were imported
  normally at `933daac0014` and `b7fe68c10aa`. R31 revalidates the same owners
  and remaining consumers on frozen main before D1–D7. No matrix cases yet.
- Old query-owner and absorbed lock-repair worktrees were archived/removed by
  concurrent cleanup. Active private source is
  `codex/oct7-substrate-managed-path-r27`; frozen execution and immutable evidence
  are separate. Worktree removal does not imply proof or absorption.
- Mandatory GOD/changed Honesty retain recorded RED states. The R25 missing
  contract-ports override pin is repaired with narrower allowance. The unrelated
  analysis-probe FFI pin remains separate. Whole live-tree unsafe scan stopped
  on another writer's missing tracked input; that stop is not reported GREEN.
  Native/installed product proof and Rename Apply's missing production
  response/replay issuer remain separate.

## Integrated repairs

All SHAs below are local Semantica main ancestors. No remote publication is
claimed.

| Checkpoint | Before → after | Focused evidence |
| --- | --- | --- |
| `933daac0014` — managed TARGET path | Invalid String operations on PathBuf → one allocation-free native-byte validator, checked work addition and original-control charge before exact path comparison. Lossy/component aliases retain the existing typed phase-specific refusal. | R30 new path module 5/5, zero ignored. Positive UTF-8 and four rejection/precedence controls executed. Two paths imported with normal main coordinator/hooks; foreign index preserved. |
| `b7fe68c10aa` — parse test imports | Extracted test modules retained duplicate imports and old sibling/control paths → refer to the existing canonical owners, plus explicit new child-module file path. No existing assertion or production mapper body is weakened. | R30 error mapper 4/4, parse 15/15 and file identity 6/6, zero ignored. Three test import paths imported normally after their whole affected modules passed. |
| `b453a31eb84` — cross-source seed closure | Metadata with `--no-deps` omitted local `[patch.crates-io]` dependencies; a cloned tree-sitter build script retained the removed donor path → full locked/offline resolution admits all local packages to existing cleanup, while registry artifacts remain reusable. No vendored code or compiler flag changes. | Complete sanitizer owner 13/13 passed, no skips. Actual donor-removal regression produces output `new` after repair; the original sanitizer fails with a missing donor path. Normal main coordinator/hooks; foreign index retained. |
| `931b2ed9db8` — lock closure | Current manifests added existing internal dependency edges while root/nested lock rows stayed stale → add only 13/11 missing edges. No new package or version change; two foreign working dependency additions remain outside this commit. | Both private-source locked/offline metadata diagnostics passed. Normal main commit attestation is present. R25 actual Native minimal reached RED101 before tests; R26 then advanced to the serializer import failure; consumer acceptance remains source-bound. |
| `400db0750c2` — port policy | A module-wide unsafe allowance was unpinned → two explicit contract-method allowances plus test-only allowance, pinned by exact method-file hash. Root remains deny unsafe; every signature/body/precondition is preserved. | Six boundary policies and canonical scoped inventory passed. Eight original-control custody cases remain pending on R31. |
| `e73d3274f80` — serializer compile closure | The native minimum rejected redundant `use serde::Serialize` under `-D unused-imports` → remove that one import inside its existing trait impl. Tuple/framing and method body are otherwise identical. | Actual R26 failure is retained with zero executed cases. Native minimum requalification is active on R30; R31 freezes the integrated main tuple. |
| `fdda8a5e997` — F6 | Client factory/cache publication could race terminal close → open check, factory and cache insertion share the lifetime lock; close invokes callbacks outside it; failed factories publish nothing. | Complete affected Python owner: 12 passed. Original close-first behavior fails the independent regression. |
| `5d3f87f47ab` — query production | Generic query owners referenced a feature-gated GQLang JSON control and lacked production imports/dependencies → existing JSON control moved byte-for-byte to neutral source-issuance owner; old GQLang path reexports it; scoped dependencies/imports/rollback bounds repaired. | Frozen QBC production check passed. This is compilation evidence, not PTA or product execution. |
| `457868f1c4f` — F10 | Agent requests could use a different root and ignored file/budget restrictions → startup captures one canonical directory; request admission rejects root mismatch, authored file lists and unknown budgets before SDK work. | Affected API/lifetime suite: 37 passed. Original wrong-root HTTP success fails the regression. |
| `0c3c26c90bc` — F5 | Public apply-changes discarded degradation, API emitted `ok=True`, projection lost native codes → SDK preserves typed outcome plus code/detail reasons; API reports degraded `ok=False` with applied data. | 43 affected cases passed, no skips. Three independent original-behavior regressions fail as expected. A broader unchanged suite exposed real Rename producer drift plus transport fixture drift. Rename request drift is repaired below; that earlier broad suite is not claimed green. |
| `975d1a3a41a` — F7 engine | Caller timeout ignored → one absolute deadline across existing SCC/worklist/narrowing and final result publication; lowering keeps typed stop and adapter covers all three domains. | Whole owning libraries: engine 73, lowering 58, adapter 21 passed; no filters or skips. |
| `e46cf07f5b1` — F2 Python | Current closed contract requires cap on every phase; SDK omitted it and Apply exposed no caller limits → Preview/Rollback retain null; Apply takes explicit typed positive u64 output/live limits and rejects absent/invalid limits before Preview. | 321 cases across the SDK execute owner and new cap/phase contract tests passed, no skips. Original Preview producer fails generated contract admission. |
| `8edd1b0733c` — F4/F9 Q3 | Cap was checked after neighbor insertion; composite slices retained BFS discovery direction and could accept bounded prefixes → admit before insertion, typed Incomplete, original induced eligible PDG edges. Empty-PDG failure precedence is retained. | Fresh frozen `a620b157998`: Q3 module 37/37 passed. Original cap oracle executes 1 and fails with root incorrectly admitted at cap0. Runtime projection acceptance remains separate. |
| `ecf12903bf3` — F7 native wire | Deadline stop lost its finite error/category contract → append payload-free RuntimeError stop, preserve old wire tags, map to AnalysisTimeout/ResourceLimit and close minimal serde feature. | Wire integration 2/2 and timeout/category projection 1/1 passed on their frozen owner sources. Concurrent main PTA feature ownership change is preserved; whole latest-main qualification is not claimed. SDK caller conversion remains a separate candidate. |
| `7bbdcc1d4e3` — dependency owner | Shared generation result was stranded in a Java-only child; row iterator and semantic view lifetimes were insufficient; backing import was narrower than its unconditional binding users → restore one shared result owner and exact source borrows/import closure. | Frozen `1cfc4c41d7a`: canonical Java, Python-only and cfg-dfg owning library checks all passed. Six reviewed paths integrated; no new calculation or dependency. |
| `4518fd6fec5` — Parser cfg | Java raw helper and raw-error facade compiled outside their active caller feature set → both mirror the exact existing caller predicate. | The same three owner checks passed. Only two cfg hunks integrated. Every foreign working/index change was retained; no Java feature was enabled to make Python minimum pass. |
| `0dd691a651c` — analysis test closure | Engine-only test reexport escaped its feature, and 14 ordinary CFG fixture inputs omitted the new source-control field → mirror the defining module predicate and the existing ordinary production constructor. | Frozen `9b5ea151425`: complete affected CFG owner 47/47 passed, zero skips. Runtime Slice owner also compiled and executed on the same tuple. Four paths integrated first. |
| `1a5cb6b0d29` — F4/F9 runtime | Bounded prefixes could publish success; PDG discovery pairs fabricated CFG direction/kinds; removed Dice statements retained occurrence metadata → typed partial refusal, finite-bound recomputation, original CFG edges on final membership, included occurrence metadata with retained parameter provenance. | R14 canonical Slice helper 6/6 passed, zero skips; all four independent output/membership regressions executed. Five paths integrated after the verified dependency checkpoint. |
| `375173d94dc` — receipt visibility | Four internal decoder targets/seeds were public inside a private module, failing minimal compilation → keep those targets, seeds, constructors and internal reexports crate-visible; public generic decoding API and wire schema unchanged. | R15 complete minimal receipt owner 69/69 passed, zero failures/skips/nonselected; shape parity and malformed-input controls executed. Three paths integrated with normal hooks. |
| `669a4608343` — locked dependency closure | Committed parser/source-transition/symbol/executor manifest edges and Python vendor patch were absent from outer/inner lock rows → reconcile exactly five package rows per lock, retaining foreign changes and package versions. | Same committed source/siblings: full outer and inner `cargo metadata --locked --offline` each exit0. Dependency resolution proof only; Rust owner execution follows separately. |
| `00a54727bae` — event extraction and phase oracle | Extracted free helpers retained `Self` and redundant mutable reborrows; the new observation test skipped canonical lease commit → qualify original enum, reuse original mutable hasher, and assert Visible then Committed after original lease commit. | Whole event owner 173/173 passed with zero ignored/filtered cases; identity/terminal/replay and exact phase witnesses executed. Only two paths imported with normal hooks and claim/barrier release. |
| `8fe0dc8ca3a` — original lifecycle interface | Getter moved across crate ownership remained private, identity-control import was missing, and an internal root identity was unnecessarily public → expose the original immutable borrowed lifecycle, import canonical control, and keep root identity crate-visible. | Canonical JSON integration module 13/13 passed (24 other target cases filtered by module), zero skips; original lifecycle/root, cancellation, exact work and memory cleanup controls executed. Three paths integrated normally. |
| `33ff8c41b3b` — source queue absorption | Six-file quarantine repair, SDK timeout conversion and two-file Rename binding/proof source match the reviewed candidate exactly. | Source absorption verified again at main `dd7aa33b2b9`; quarantine 130/130 at R24, SDK compile blocked, Native NOT_RUN. Adoption creates no duplicate commit. External absorption hook/coordinator attestation was not observed; this records ancestry/bytes, not final qualification. |

### Code owners

Paths are relative to `semantica-codegraph-v2/`.

- F6/F10: `apps/api/api/dependencies.py`, `main.py`, `routes/agent.py`.
- F5: `packages/sdk/python/codegraph_sdk/client.py`,
  `_product_apply_changes_projection_v1.py`, `apps/api/api/routes/indexing.py`.
- Query production:
  `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/source_issuance_control_v3/json_projection_v3.rs`
  and `quanta-runtime-query-executor-owner/src/query_executor/` consumers.
  The calculation is retained; moving its module does not add a second owner.

## Earlier frontend/consumer actions — source recheck

At committed Semantica `60211acdb1a62d0ae5f18d677211cf1c4b63c63a` and QGLang `c3b84ab`, the earlier
Graph contract, golden and EXISTS/conformance code repairs are present:

- Public ingress selects the bounded product Graph profile; host validates
  witness cap1, Reachability Endpoints and Path NodeSequence. Independent public
  family fixtures assert cap1 for both kinds.
- Consumer golden asserts contract/qualification schema8 against the current
  frontend vector. Source pin/gate presence is not execution qualification.
- The mixed EXISTS owner contains both false first-term and false second-term
  nonempty-input cases with independent expected empty results.
- Frontend conformance includes Rename; authored result shapes remain explicitly
  rejected at sema. Document/execution are split into owning modules, and local
  static composes size-lint plus clippy-msrv.

These are current-source observations, not new code written by this recovery or
new downstream runtime verdicts. Retain actual product proof in the remaining
index instead of reintroducing those repairs.

## Remaining repair candidates

Native Rename and the SDK deadline caller are absorbed source changes
on `codex/oct7-substrate-query-owner-build-r1`, after the three-file private
receipt-decoder visibility prerequisite was integrated at `375173d94dc`. Runtime
Slice and its feature/input dependency are integrated above.

| ID | Structural change | Acceptance still required |
| --- | --- | --- |
| F2 native | Add the missing canonical `mutation.refactor-rename` runtime binding. Retain existing typed backend, provider checks and Preview/Apply controls. Exact mutation inventory is the oracle, not a refreshed numeric count. | Execute native binding parity and feature-off refusal; prove public Preview reaches existing dispatch/backend; preserve phase/cap/host refusals. |
| F7 native | Solver/lowering/adapter are integrated. Append a payload-free RuntimeError deadline refusal and preserve it through native SDK plus exhaustive timeout/category mappings; do not invent QueryId. | Engine owner: 73 passed; lowering owner: 58 passed. Whole adapter owner: 21 passed. Native wire integration 2/2 and timeout/category projection 1/1 passed on their recorded frozen sources; SDK projection proof remains pending. Check default30s, iteration distinction, all domains, zero/past deadline and non-timeout rejection controls. |

F7 does not promise hard interruption inside user callbacks, collection sorting
or lowering/projection calls. The deadline is checked at cooperative work
boundaries and after those calls; an overdue result is never published as success.

The integrated native wire stop appends `AbstractInterpretDeadlineExceeded` as
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
  pending consumer work was superseded again by the current R12 source.
  Cancellation supplies no test verdict.
- Rename provider proof now additionally enters the actual admitted context,
  registry and worker before the existing backend. Its source is `7eea70e4a06`;
  an isolated handler receipt alone does not close that path.

### Compile recovery and current acceptance source

Historical compilation failures executed zero tests. They are retained as
failure evidence, not as error counts for current main.

| Attempt | Reached failure | Canonical repair / successor |
| --- | --- | --- |
| R6 Runtime Slice, `a620b157998` | 84 PTA/call-graph errors under `pta-rail-test-support`; that class did not enable `slice-runtime` and could not validate Slice. | Actual Slice owner class is `quanta-runtime-analysis.no-default.8a9cf0054bb7`, features `cfg-query-test-support,slice-runtime`. Separate PTA acceptance remains open. |
| R7 joined Rename Preview, `7eea70e4a06` | 12 PTA adapter errors, including a shared Java/Python generation result stranded in a Java-only child. | Shared generation owner, explicit Python binding import, exact row iterator lifetime and retained semantic view; six-file dependency checkpoint `7bbdcc1d4e3`. |
| R6 SDK deadline, `a620b157998` | Three references to `NativeLoweringOutputBackingV1` failed because its import was narrower than its unconditional binding users. | Existing backing type imported on all active binding compile paths; part of the same six-file checkpoint. No new feature or dependency. |
| R8 Python-only / R9 `cfg-dfg` | Ungated Java helper referenced an absent augmentation type; raw-error facade had no active consumer. | Two items mirror the existing caller predicate, integrated at `4518fd6fec5`. Foreign Parser working/index bytes were retained. |

All three R10 canonical Java, Python-only and `cfg-dfg` adapter checks passed
on `1cfc4c41d7a`, digest
`435e5a731ce4225299569a60052ac8d97d0f93da258289fb15725aa8c5a6d5ba`.
These are complete production-library compile checks, not runtime tests.
Both dependency checkpoint claims and barriers were released normally.

The queued R10 consumer jobs were cancelled before completion to adopt newer
committed interprocedural/semantic repairs; cancellation supplies no verdict.
R11 on `462fe9bb5c66831e522a4cfcf63292a42aa11aa3` reached one Parser
unused recoverable-health import error and executed zero Slice tests. That
repair was already committed on main at `cf5f5c76503`; no duplicate repair was
authored. Its failure receipt is archived; pending R11 jobs were cancelled
with no verdict.

R12 source `d2861f04395528d5b9753c71391d9ccb4e12362b` adopted that
committed Parser repair and reached Runtime analysis test compilation. It
executed zero tests: 15 errors comprised one incorrect StatementId import in
the new regression, one test facade exporting an engine-only module without
its feature, and 13 compiled CFG input literals missing a newly added field.
The original failure receipt is archived; pending jobs were cancelled without
a verdict.

The Runtime Slice/CFG acceptance source is
`9b5ea1514259354aaa8cfbddb07d12ad95a8021e`,
digest `0550b5ffe05374d689f0d21ffddeee3c3bc31485a3c13b3a56a227f7f65491db`.
It retains the committed main input `2de55f286588953de2b2d017d21d930f59fdc8c2`
and adds these scoped repairs:

- The regression imports StatementId from its canonical existing analysis
  contract, matching the actual projection owner; no dependency is added.
- The test-only recovered-role reexport requires its defining module's
  `interprocedural-engine` feature. No feature is enabled to mask a failure.
- All 14 affected CFG fixture input literals explicitly set
  `current_source_control_v3: None`, matching the existing ordinary-profile
  production constructor. Controlled sessions continue to obtain their actual
  source control through the existing session callback and fail closed when
  unavailable. This migration neither issues fake authority nor relaxes that
  production boundary.

The first R13 selector used a public facade name and selected zero tests.
QBC rejected it with exit86 and RED; its archive records 0 passed / 1573
nonselected. The source compiled, but that run supplies no runtime proof.
Remaining queued jobs were superseded without a verdict.

R14 reuses the exact same frozen source and actual canonical Rust modules:
`queries::advanced_owner::slice_query::slice_query_helpers` includes both
output and occurrence regressions;
`queries::intraprocedural_owner::cfg_query::tests` owns the CFG input cases.
They passed 6/6 and 47/47 respectively, with 1567/1526 nonselected cases and
zero skips. The SDK attempt and native successor are recorded below.
Exact witness names,
nonzero pass/zero failure/zero skip counts, archive hashes and source digest
are checked before each import.

The four-file analysis feature/input dependency checkpoint must pass its own
CFG owner and reach local main before the five-file Runtime Slice consumer
checkpoint. This preserves compilation closure at each checkpoint. Native
Rename and SDK deadline imports retain their independent acceptance guards.
Concurrent Parser working changes remain excluded from the frozen source.

The old `OCT6-SUBSTRATE-SCC` write claim was released after confirming no
unintegrated candidate delta in any of its 24 paths. This retires write
ownership; D2 consumer/product qualification is still open.

### SDK receipt visibility and actual distribution proof

R14 SDK deadline failed during receipt-wire dependency compilation with 10
`unreachable_pub` errors; zero SDK tests executed. The original failure archive
is retained. Remaining R14 native requests were cancelled without a verdict.

The four reported decoder types have no external callers and already live in a
private module. The repair keeps the owned target and nested command/hop/partiality
seeds plus constructors/reexports `pub(crate)`. Public generic decoder targets,
SDK receipt seed, schema and wire bytes are unchanged. No public API or decoder
implementation is added.

The previous providers-only class (`provider-gqlang,provider-mutation`) uses the
Interactive distribution manifest. It cannot establish positive public Mutation
support. R15 uses the registered minimal feature-off class and actual `dist-full`
class `quanta-cli-ingress-runtime.no-default.9d4f950584d2`; whole registry and
mutation owners replace a provider-only claim or isolated Preview result.
Existing Apply no-effects/absent-issuer refusals remain required witnesses.

The stale five-service numeric count test is removed. The surviving canonical
contract-to-registry exact ID/service map, duplicate checks and independent six
native Mutation ID oracle detect omissions without another drifting number.

R15 candidate is `e3848ef6dd35406d6c9016929236215e1b4b0d0d`, source digest
`5e4c3d025dacc807e99fb356b6e96f96cef6604561d615153f5b6f19da6c8085`.
The complete minimal receipt owner passed 69/69 with zero failures, skips or
nonselected tests, and three files reached main `375173d94dc`. SDK conversion
failed before tests on one current quarantine digest unused import; pending
native jobs were cancelled with no verdict. Their PASS is not claimed. Its committed sibling pins remain
QGLang `68ff02ef443f` and Index `6a3f6afc8c28`, with foreign dirty excluded.

### Current-main adoption and quarantine prerequisite

R15 SDK reached `quanta-sdk-exact-repo-quarantine-owner::record_digest_v1`
and failed on its unused `std::fmt::Write` trait import. The digest writer takes
`impl std::fmt::Write`, so the trait is already supplied by the bound; its body,
field order and separators are unchanged by removing the unused import. The
failure archive contains zero SDK tests. Receipt69 remains valid at its source
and integrated three-file checkpoint; pending native cancellation is no verdict.

R16 source `20bdc8ea84c8ce165d78c3c2bb4e52cc4dd0d7a9` adopts committed
main `522c94ac2648b4727f77f9829f33d57c55d6f26c`, digest
`8609bd5080e8ad96870968eb5000d5d204ba70cc91b4a0090ef9c1c4a212ddc7`.
The candidate's complete difference from that main input is exactly four files:
quarantine import1, SDK deadline1, native Rename2. All earlier repairs are already
in the adopted main source; no foreign dirty bytes are included. Whole quarantine
owner, SDK conversion and the minimal/full native owners retain separate guards.
The quarantine owner checkpoint must precede SDK integration.

### Locked dependency closure and current retry

R16's first quarantine owner was rejected by `--locked` before compilation or
tests. Separate full Cargo metadata diagnostics on archived committed source and
the same pinned siblings identified five package rows in both lockfiles:
`quanta-adapters-parser`, `quanta-contract-source-transition`,
`quanta-java-binary-symbol-authority`, `quanta-sdk-runtime-executor` and
`tree-sitter-python`. Seven declared dependency entries per lock were missing;
the already committed Python vendor patch still carried registry/checksum fields.
No package/version set changed. Metadata with `--no-deps` does not establish
resolution; the recorded acceptance uses full metadata with `--locked --offline`
for both workspaces, each exit0. This is dependency proof, not a Rust verdict.

Only those five rows in each lockfile were reconciled into main `669a4608343`.
Other committed rows and all foreign working/index work, including separate
PTA and compression-library changes, are retained. Normal hooks and coordinator
completed and the lock write claim/barrier were released.

R17 source `497bfea5a4d7ce9387b26293c1d8287c862a75ef`, digest
`124cd4a26ccb4d1f659e321ae3e202908c87ec6e3d1c701392186976b90b4e55`,
adds those generated locks to the R16 candidate. Quarantine and SDK reached
RuntimeExecution but stopped before tests on four E0624 errors and a dependent
E0282 error: the relation control moved to SourceAuthorityWire in committed main
`7681aa7a875`, while its existing `lifecycle_v3` accessor remained crate-private.
The JSON terminal consumer remained in RuntimeExecution and therefore lost
access. Both failure receipts are archived; remaining native jobs were cancelled
without a test verdict. This was a module-move interface defect, not a JSON or
budget calculation failure.

R18 exposes the existing immutable borrowed lifecycle at its canonical owner,
with no replacement lifecycle, changed authority, wire schema or allocation.
The original lifetime is retained. Existing JSON terminal regressions cover
original/reconstructed lifecycle, distinct zero-sized lifecycle, unrelated root,
cancellation/timeout, allocation checkpoints and nonrefundable work. The actual
integration target is `gqlang_relation_execution_v3`, module `json_projection`,
under the registered relational-composition feature class. It precedes whole
quarantine, SDK conversion and minimal/real Full native owners. Acceptance and
main integration remain pending until their source-bound receipts are green.
R18 also exposed four missing canonical identity-control import uses in
`gqlang/composition_plan_v3.rs` when the relational feature was activated. Its
compiler log is diagnostic only: a concurrent model inspection imported frozen
QBC modules without Python cache suppression, changing protected directory
metadata and invalidating exact custody. No R18 owner success is claimed. The
invalid-custody record and logs are archived; later requests were not executed.

R19 adds only that canonical import to the R18 repair. Python cache writes are
disabled for all frozen-module inspection (`-B`/`PYTHONDONTWRITEBYTECODE`). The
existing semantic owner remains unchanged; no separate identity implementation
is added. All six Rust candidate paths pass the impacted boundary preflight;
actual source-bound tests and main integration remain pending. The 45 D1–D7
owner requests were also reprojected on R18 with unchanged requests/commands;
that result is resolution only and requires rebinding to R19 before execution.

### R19 budget facade visibility and R20 successor

R19 has a canonical archived compile failure, zero tests: the now canonical
`ExecutionMaterializationRootIdentityV3` import was `pub` in private
`budget_handle.rs`, although the crate root does not export it. It is used only
by the internal exact-mutation binding. R20 makes that import crate-visible;
all existing public budget exports remain unchanged. The lifecycle/composition
boundary checkpoint therefore consists of exactly three files.

The configured six-path GOD gate is RED (new27/worsened1 against its stored
baseline). Running the same canonical scanner against reviewed committed main
`522c94ac2648` versus R19 proves introduced findings0 and worsened0 in this
candidate. Those recorded failures are preexisting source/baseline debt, not 28
new defects introduced by the repair. The gate remains RED; no baseline, allow
or threshold was changed. Current-main acceptance still needs its policy owner.

### R20 event dependency and R21 proof oracles

R20's canonical JSON target stopped in `quanta-change-event-contract` with four
compiler errors and zero tests. A fingerprint helper extracted from an enum impl
retained three `Self` variant names in a free function and attempted to borrow
an already mutable hasher parameter as `&mut hasher_v1`. The current main working
file already has the exact repair. R21 adopts only that reviewed file, reconciling
`OCT7-MUTATION-RECOVERY-SOURCE-CUSTODY`; it is not claimed as newly authored
fingerprint logic. All tags, hash framing, field order and schema bytes remain.
The complete event library must execute its independent identity, terminal and
replay controls before its one-file main import.

Changed-scope Honesty on R20 completed: 11 gates passed, five failed (shape-only,
relationship, determinism, registry-honesty, unverified-early-success), with zero
bad invocations. The new non-timeout SDK regression had a real weak oracle:
partial `contains` assertions. R21 compares the entire typed error and exact
expected value for non-timeout, missing authority, invalid input and cancellation.
The SDK shape-only scan now reports only the unchanged
`empty_pipeline_dfg_propagates_direct_recovery_error`; all four strengthened
operation-error cases have no shape-only finding. The configured gate remains
RED on that residual case; no blanket baseline update is performed. Remaining
Honesty reports require source-specific attribution, not conversion into alleged
runtime defects by raw report count.

### Residual code map and completion conditions

### R21 terminal-phase oracle and R22 successor

The complete event library compiled and executed 173 cases on R21: 172 passed,
one failed, no ignored or filtered cases. The new executing-observation test
expected `PublicationAlreadyCommitted` immediately after the irreversible
publication callback. The canonical owner explicitly records
`PublicationVisible` there; only `OperationPublicationLeaseV1::commit_v1`
records `PublicationCommitted`. An existing independent materialization test
already proves both states. R22 corrects the new test to assert Visible,
commit the original lease, then assert Committed. No production state transition
or failure contract changed. The R21 receipt and failure logs are archived;
the six later requests were cancelled without a verdict. Event acceptance
requires the full owner and this phase witness on R22 before two-file import.

The shared-parent writer remains canonical change-event owner; JSON reads its
original lifecycle, SDK projects its existing finite errors, and Rename retains
existing dispatch/backend contracts. No alternate observer, issuer or solver is
introduced. Exact paths, source closure and source-bound receipts are recorded
in this report and each guarded import artifact.

### R23 quarantine test migration

R22 quarantine failed before running any cases: six unresolved test `Arc`
references, one test call to removed `prepare_terminal_identity_before_apply_original_v3`,
and one unused local test trait import. R23 imports Arc in the test module,
uses the existing `prepare_terminal_identity_before_apply_controlled_v3`
with the same original control, and removes the unused import. These three
one-line repairs preserve the poison test's exact first-refusal and unpublished
slot assertions. No compatibility alias, fabricated issuer or new preissuer
implementation was added. The existing record-digest import cleanup joins these
three files in one four-file owner checkpoint. Required witnesses include
poisoned slots, reused committed storage, cancellation preserving pending state,
digest tamper and exact durable response replay. All four still require actual
whole-owner success before import; compiler success alone is insufficient.

Event 173/173 and JSON 13/13 remain R22 results at their recorded source. Their
five owning paths did not change in R23; already-integrated checkpoints are not
replayed or relabelled as new R23 runs. The remaining five owner requests and
45-request matrix bind to the R23 source/digest above.

### R23 reachable preissuer refund defect and R24 repair

R23's full quarantine library executed 130 cases: 129 passed and one failed,
zero ignored or filtered. The existing native-birth failure matrix detected
40 live intermediate bytes remaining above the original baseline while the
fingerprint stayed alive. `original_reason_admission_v3` reused the fingerprint's
long-lived memory group. If the reason String was born but its final retained
owner birth failed, dropping temporary values could not release that String's
charge because the fingerprint still retained the group.

R24 keeps original marker/group validation, then asks that same control for an
independent owned group through the existing `OriginalSetupAdmissionV3::try_new_v3`.
Failure drops that group; successful reason custody keeps it through its last
clone. The only old `try_from_group_v3` caller is retired with its now-unused
private constructor. No new budget/root, issuer or allocator contract is added.
The existing ten-point allocation-refusal regression keeps its exact zero
residual-memory expectation; a diagnostic labels the refused birth index.
Existing detached-reason retention, poison/refusal and replay controls remain
mandatory. Actual whole-owner success and source-only main import are pending.
This is a reachable resource-lifetime defect, separate from the earlier test
compile migration; R23 RED is not rewritten as a passing result.

Paths below are relative to Semantica. Each group is imported independently
only after its own acceptance; historical receipts are not composed into a
whole-repository result.

| Group | Exact owning files | Completion condition |
| --- | --- | --- |
| Analysis test feature/input closure — complete | `packages/analysis/quanta-v2/crates/quanta-runtime-analysis/src/queries/`: `interprocedural.rs`, `intraprocedural/cfg_query_tests.rs`, its `reuse_authority.rs` and `unreachable_policy.rs` children | Complete CFG owner 47/47 and Slice helper 6/6 passed; four paths integrated at `0dd691a651c` before the Runtime Slice consumer. |
| Runtime Slice — complete | `quanta-runtime-analysis/src/queries/advanced/`: `slice_query_bridge.rs`, `slice_query_helpers/output_materialization.rs`, `slice_query_helpers/output_final_membership_tests.rs`, `slice_statement_projection.rs`, `slice_statement_projection/occurrence_projection.rs` (crate under `packages/analysis/quanta-v2/crates/`) | Actual canonical helper 6/6 passed with `slice-runtime`, including all four direction/kind/multiplicity, partial and membership regressions. Five paths integrated at `1a5cb6b0d29`. |
| Receipt private visibility — complete | `packages/analysis/quanta-v2/crates/quanta-runtime-receipt-wire/src/`: `canonical_receipt_seed_v1.rs`, its `receipt_gqlang_seed_v1.rs` and `receipt_nested_seed_v1.rs` children | Whole minimal receipt owner 69/69 passed; three-file prerequisite integrated at `375173d94dc` before the SDK caller. SDK conversion retains separate acceptance. |
| Event fingerprint extraction and phase oracle — complete | `packages/analysis/quanta-v2/crates/quanta-change-event-contract/src/ports/`: `ide_source_lifecycle_port/identity_fingerprint_v1.rs`, `operation_context.rs` | Adopt the reviewed helper repair; test Visible before lease commit and Committed afterward. Whole event library and identity/terminal/replay controls; production framing, tags and transitions remain unchanged. Whole owner 173/173, source-only two-file main checkpoint `00a54727bae`. |
| Relation lifecycle interface — complete | `packages/analysis/quanta-v2/crates/quanta-runtime-source-authority-wire/src/original_source_control_v3/relation_control_v3.rs` | Existing JSON projection integration module executes all original-root/lifecycle, cancellation/timeout and allocation controls; immutable accessor visibility plus canonical `gqlang/composition_plan_v3.rs` identity-control import and internal `budget_handle.rs` root-identity import, no owner or schema duplication. Canonical module 13/13, main checkpoint `8fe0dc8ca3a`. |
| Quarantine reason custody and test closure — source absorbed | `packages/analysis/quanta-v2/crates/quanta-sdk-exact-repo-quarantine-owner/src/`: `record_digest_v1.rs`, `committed_terminal.rs`, `publication_lifecycle.rs`, `hellgate_tests.rs`, `original_setup_admission_v3.rs`, `publication_lifecycle/terminal_preissue_v3.rs` | Release failed reason storage independently of a live fingerprint, using the same original control. Retain original group validation, remove the now-unused shared-group constructor and migrate tests. Whole owner 130/130 on R24 includes all 10 allocation-refusal points, last-clone retention, poison/storage/cancellation/durable replay witnesses. Six-file source absorbed at `33ff8c41b3b`; current-main proof remains separate. |
| SDK deadline — source absorbed, proof blocked | `packages/analysis/quanta-v2/crates/quanta-runtime/src/sdk/abstract_interpret_builder.rs` | Native finite timeout survives actual port conversion; non-timeout errors retain their prior rejection. One-path source absorbed at `33ff8c41b3b`; actual SDK tests are blocked by the contract-port dependency. |
| Native Rename — source absorbed, proof pending | `crates/quanta-cli-ingress-runtime/src/dispatch/operation_binding_registry_v1.rs`, `dispatch/mutation.rs` | Exact inventory plus minimal refusal and real `dist-full` admission; whole registry and mutation owners including admitted context → registry → worker → Preview backend. Both source paths are absorbed at `33ff8c41b3b`; actual native proof remains pending. Production Apply remains unavailable until its real issuer exists. |
| PTA dependency repair — complete | `packages/analysis/quanta-v2/crates/quanta-adapters-pta/src/`: `production_live_native_workspace_generation_v1.rs`, its `original_java_workspace_v1.rs` and `original_java_evidence_v1.rs` children, `production_live_captured_semantic_workspace_generation_v1.rs`, `production_native_lowering_relation_output_v1/admitted_storage_v1.rs`, `native_function_build_generation_authority_binding_v1.rs` | Three canonical feature owner checks passed; six-file delta integrated at `7bbdcc1d4e3`. |
| Parser feature closure — complete | `packages/analysis/quanta-v2/crates/quanta-adapters-parser/src/language_host_adapter/semantic_module_v3/core/mod.rs`, `semantic_module_v3/mod.rs` | Three canonical feature owner checks passed; only two cfg hunks integrated at `4518fd6fec5`, retaining all foreign working/index bytes. Later retirement of legacy callers may remove or narrow those items further. |

## Final acceptance order

1. Complete independent owner regressions and preserve typed failures.
2. Import only reviewed, owned paths into local main with normal hooks and
   shared-main coordination; recheck drift immediately before import.
3. Retire only source-confirmed integrated repairs from the remaining-work index.
4. Execute affected mandatory consumer/policy gates on one frozen main tuple.
5. Execute actual native daemon/product acceptance. Installed Linux CHECK still
   needs release-owned signed immutable slots and independent approval; helper
   tests and compiler success do not supply those prerequisites.

## Source-bound proof handoff

Historical blockers are preserved in immutable archives: SDK R24 stopped on
unsafe/forbid declarations; Native R25 stopped on committed lock drift; Native
R26 stopped on a redundant generation-row import. All three executed zero cases.
The current source includes the narrow policy fix, root/nested lock closure and
single-import removal; these source repairs alone do not establish execution.

R27's Native and quarantine owners stopped on four PathBuf type errors, with
zero executed cases. R28's managed-path owner stopped on the new child module
path plus existing parse/identity imports, also with zero executed cases.
R30 binds private candidate `7ba2d42e31a66271d114f212484104d2ce721882`, gqlang
`7ff7ed8290ed872863daa6e83cdb01b338c27160` and Index
`8642fa9b3b47ac58e4b3d9f2feaea66c599be29d`. It is a scoped candidate proof,
not qualification of moving main. Public custody methods retain actual producer
birth admission, original group/operation identity, borrowed row containment and
derived-allocation obligations; only their lint scope narrows.

### Cross-source build seed — repaired owner

R29 imported a cached tree-sitter build script that embeds compile-time
`CARGO_MANIFEST_DIR`; its donor snapshot was removed. The destination's source
files were present, but the executable read the old absolute path. Existing
sanitization enumerated workspace members and explicit dependency paths from
`metadata --no-deps`, omitting patched registry dependencies such as tree-sitter.
This is build-cache correctness, not a parser/product behavior defect.

The sanitizer now uses full locked/offline metadata and starts from packages
whose Cargo `source` is null. Existing cleanup and verified clean-cache shape
reuse remain the owners. The change covers all resolved local patches, not an
exception for one library. Metadata resolution reads more package records;
registry compile artifacts remain reusable and resolution still fails closed.
The actual temporary-Cargo fixture deletes the donor, exposes the stale script,
and verifies the rebuilt destination prints `new`. All 13 sanitizer cases pass.
Normal main checkpoint: `b453a31eb8470771c8cb81c8b4613277923d7384`.

### Managed TARGET path — integrated scope and acceptance

Owner: `quanta-runtime-query-surface-foundation/src/parse/managed_target_v3.rs`.
Consumer: ParseQuery pre-key admission, execution and cache admission for the
original retained TARGET. Two invalid String-style PathBuf checks become one
private helper that borrows native encoded bytes, checks work addition, charges
the original control before comparison, and returns the existing phase-specific
`RUNTIME.GRAPH_PARSE_TARGET_PATH_MISMATCH_V3`. No String allocation, lossy path
conversion, component normalization, new authority or public contract change.

Integrated files: the managed-target owner and its `path_binding_tests_v3.rs`
child, plus test-only import repairs in `file_text.rs`, `parse.rs` and
`parse/parser_port_error_boundary_tests_v3.rs`. Production parser mappings are
unchanged. Old imports are redirected to their existing owners; test assertions
and warning policy are retained.

Executed R30 acceptance: five new cases (exact UTF-8 in both phases; different member; component
alias; original work refusal precedence; non-UTF-8 versus replacement-character
alias) passed with zero ignored cases. Existing parser error mapping 4, parse 15
and file identity 6 also passed. Main checkpoints `933daac0014` and `b7fe68c10aa`
preserve foreign source/index bytes through the normal coordinator. R31 binds
those integrated files and later common-owner changes to one main tuple; consumer
execution remains a separate acceptance boundary.

| Next action | Acceptance |
| --- | --- |
| Revalidate integrated path/parse/identity owners, Native minimal, original output-custody eight, JSON/quarantine, SDK five controls and Native Full registry/mutation on R31. | Exact owning targets/features; nonzero selected cases, zero failures/ignored; original identity/refusal/refund, exact typed errors, literal inventory, provider-off refusal, admitted Preview and Apply refusals. |
| Execute D1–D7 owner45 on the same source. | Preserve PTA-first ordering, reviewed witnesses/minima and ignored policies. Resolve-only maps and compile success do not count as execution. |
| Close attributable mandatory policies and actual product acceptance. | Retain unrelated GOD/Honesty/FFI/input findings until their owners resolve them. Native daemon/installed acceptance stays separate; Linux CHECK requires signed immutable slots/independent approval. |

Evidence root:
`/Users/songmin/.codex/task-evidence/oct6-substrate-dedup/recovery-20261006/query-owner-build-r1`.
`final-source-absorption-review.json` is the historical main `dd7aa33` nine-path
observation. `lock-r25-r2-import-state.json` and
`port-policy-and-import-r26-commit-state.json` record the normal new checkpoints.
`remaining-r31-snapshot-state.json` and its owner state/map bind integrated-main
acceptance. `remaining-r30-owner-state.json` and archives retain the actual 30
focused passes used for main import; R27/R28 failed receipts retain their original tuples. Historical receipts keep their original source roots.
Private worktree deletion is not absorption or proof evidence.

## Frontend local CI

`just ci` completed on QGLang HEAD `68ff02ef443f` with its then-current
working overlay. The runner recorded source snapshot
`f3a565f81f5d412a2aefd011319c6db0946b35f4706f786e3b321cff7b485dd6`:
all groups passed; nextest 1514/1514 with zero skips, doctest 10,
external-consumer positive 1 and negative compile-fail 23.
Subsequent edits here and to the remaining-work/ADR reconciliation are prose
updates; the receipt is retained at its original source. This is local CI,
not a clean committed-HEAD qualification receipt.

## Verified R43 checkpoint — integrated 10 paths, cursor acceptance open

Semantica main checkpoint: `26f165989053b911bf28ce8296aa9c38bea99445`.
Normal shared-main coordinator/hooks, exact saved preimages, committed postimages
and preservation of foreign staging were verified; the barrier and completed
path/CFG/manifest claims were released. No push is claimed.

| Changed owner | Resulting behavior |
| --- | --- |
| Source-authority wire, Parse TARGET and HIR | Reuse one admitted native-path byte comparator. Checked combined work, literal path components, original refusal and non-UTF-8 rejection are preserved; no copied or lossy path is created. |
| CFG checked-root entry | Backend validation remains first; a supplied current stop is checked before ordinary custody can replace it with authority rejection. A live control cannot upgrade ordinary syntax. |
| Symbol owner manifest and CFG fixtures | Promote an existing production dependency from dev dependencies and migrate three fixtures to their existing checked APIs. No package, version, feature, engine or test oracle is replaced. |

The canonical owners executed on candidate
`c55a232607c0284116e31e26d76a21150f773d6e`, based on committed main
`235cd4c48ff89bfbe8cea701dfd61ed5c93ee5db`. Source digest:
`6d9166fc6a2bedc5c891b14983c1981aa4f49fdf11f7189b8ba77bbcdaad06fe`.
Committed siblings: QGLang `7ff7ed8290ed872863daa6e83cdb01b338c27160`,
Index `8642fa9b3b47ac58e4b3d9f2feaea66c599be29d`.

| Canonical owner | Passed | Failed / ignored |
| --- | ---: | ---: |
| Wire borrowed-byte comparison | 14 | 0 / 0 |
| Parse managed-path owner | 5 | 0 / 0 |
| Whole CFG library | 413 | 0 / 0 |
| Actual relation-execution JSON integration | 13 | 0 / 0 |

All 445 passes share this candidate. Scoped GOD scan of the changed production
paths reports six scanned, zero stale or allowed entries. Test/vendor paths are
excluded by that policy; this is not a global policy verdict.

Evidence at the recorded recovery directory:
`remaining-r43-snapshot-state.json`, `remaining-r43-owner-state.json`,
`current-r43-main-import-state.json`, `current-r43-final-absorption-review.json`,
and `current-r43-owned-god.log`. Earlier R38 common 7/Java 139 and R40 quarantine
130 results retain their own tuples and are not added to this same-source total.

Exactly three reviewed cursor files remain different between the candidate and
the integration checkpoint: common checked-read walk, its traversal tests, and
vendor native cursor continuation. Keep the private checkout/branch
`codex/oct7-substrate-current-owner-r43` until the original issuer's five
memory/work/cancellation controls execute with their existing oracles.
The genuine adapter owner currently stops before cases at Java DefUse; Java and
Python still consume the retired raw-tree getter in committed main. Rust
consumer source is under a distinct active writer claim. This task has authored
no DefUse consumer body and will not restore raw getters or reparse controlled input.

The candidate includes these three private cursor files. The four owner results
therefore qualify that candidate, not the entire integration checkpoint. Native
minimum/full, SDK deadlines, D1–D7 owner45, mandatory policies, daemon/installed
Linux CHECK and clean committed-HEAD qualification retain their own acceptance.
The latest status in this section supersedes older preparation notes above.

## R44 acceptance on the integration checkpoint

The four canonical owners also executed on committed integration checkpoint
`26f165989053b911bf28ce8296aa9c38bea99445`, with no private cursor files:
wire comparison 14, Parse paths 5, whole CFG 413, JSON integration 13;
**445 passed, zero failed or ignored**. Digest:
`01074dee1916762231c18af13914ac9d133416f6eea2f373a53010d4d1c46e6a`.
Committed siblings remain QGLang `7ff7ed8290ed872863daa6e83cdb01b338c27160`
and Index `8642fa9b3b47ac58e4b3d9f2feaea66c599be29d`.

This closes the ten-path checkpoint's four stated execution rails without
using its private cursor prerequisite. It supersedes R43's candidate-only
caveat for these four rails. Actual requests, witness names, result/receipt
hashes and immutable archives are in `remaining-r44-owner-state.json` and
`remaining-r44-snapshot-state.json` at the recorded evidence root.

The three cursor paths still need the original issuer's five unchanged
memory/work/cancellation controls. Original traversal and Native minimum previously stopped before cases at
DefUse's retired raw-tree consumer boundary. Native Full and SDK deadline
execution on this checkpoint remains unperformed; consumer source writers
are active independently. No DefUse body is authored
by this checkpoint, no raw getter/reparse fallback is introduced, and no
original-issued, whole-product or installed CHECK qualification is claimed.

Moving main has subsequent commits and foreign Parse/HIR overlays. These 445
passes apply to the fixed checkpoint and committed sibling tuple above;
source-byte continuity is recorded separately in
`r44-current-main-bundle-continuity.json`. Final mandatory policy/owner45,
frontend clean-HEAD qualification and signed installed Linux CHECK retain
their separate requirements. R38/R40 results retain their original inputs.

## Latest source absorption — all reviewed source integrated

Final source reconciliation observed Semantica main
`3bf86a4735c989c051e1c143f529dc5f9b24c45e`. The three cursor postimages were already integrated by
the main queue at `dc2611b96e12aea4ebc3cb15328b528806e6b548`; all 13 reviewed
candidate postimages now match committed main exactly. The root cursor source
claim is released. This supersedes earlier statements that these files remain
private or require another source import.

Source absorption does not complete original-issued execution acceptance. The
five unchanged issuer memory/work/cancellation controls remain PROOF_PENDING
after a reached DefUse dependency compilation failure. Existing common 7 and
Java 139 passes retain R38's source; R44's 445 passes retain integration
checkpoint `26f165989053b911bf28ce8296aa9c38bea99445` and its paired pins.
No original-issued or whole-product pass is inferred from this reconciliation.

`r44-all-source-absorbed-state.json` binds exact path equality and claim release.
The private recovery checkout has no future root execution/write planned;
cleanup must preserve recovery refs/immutable receipts and independently check
process, lease, dirty and canonical reference custody. DefUse consumer writers,
Native/SDK execution and remaining owner/policy/product gates stay open.

## Additional checked-read supplier inventory — R45

Frozen source inventory: Semantica `e7bae76f70b0e771c12ac50b839e3dd8dd1abc43`. Production receivers were checked
against their signatures; this is not a count of every matching method name.

| Consumer | Retired production calls | Source entrypoints |
| --- | ---: | --- |
| Rust language-plugin | 2 | `packages/core/codegraph-lang-rust/src/language_parser_plugin/mod.rs:1414, 1465` |
| Go language-plugin | 2 | `packages/core/codegraph-lang-go/src/language_parser_plugin/parse_ports.rs:427, 478` |
| Kotlin language-plugin | 2 | `packages/core/codegraph-lang-kotlin/src/language_parser_plugin/parse_ports.rs:304, 354` |
| C language-plugin | 2 | `packages/core/codegraph-lang-c/src/language_parser_plugin/parse_ports.rs:420, 470` |
| TypeScript language-plugin | 2 | `packages/core/codegraph-lang-typescript/src/language_parser_plugin/mod.rs:1184, 1221` |
| Rust defuse | 1 | `packages/core/codegraph-defuse-extractor-rust/src/extractor/mod.rs:2526` |
| Java defuse | 1 | `packages/core/codegraph-defuse-extractor-java/src/extractor/extraction_frontdoor/mod.rs:107` |
| Python defuse | 1 | `packages/core/codegraph-defuse-extractor-python/src/extractor/mod.rs:1031` |
| Go defuse | 1 | `packages/core/codegraph-defuse-extractor-go/src/extractor/traversal/fact_helpers.rs:94` |
| Kotlin defuse | 1 | `packages/core/codegraph-defuse-extractor-kotlin/src/extractor/mod.rs:1306` |
| C defuse | 1 | `packages/core/codegraph-defuse-extractor-c/src/extractor/mod.rs:175` |
| TypeScript defuse | 1 | `packages/core/codegraph-defuse-extractor-typescript/src/extractor/mod.rs:956` |

Total: five language-plugin owners have ten retired `CfgParsedTree` calls;
seven DefUse owners have seven preparsed-carrier calls. Existing
`ParseResult::tree_sitter_tree_v1` is a different, still-declared API and is
excluded from this removal inventory, as are constructors and test fixtures.

The existing canonical T1 Rust owner receipt is compile RED, exit 101, zero
executed tests. It reaches Rust plugin lines 1414/1465 and Go lines 427/478
with E0599. Run `20261007T110706.310772Z-efeceed59a15`, digest
`feb6376939ee4dc7b103f765f404e321f2acff5a4324d762f2429b54325d1bef`; source is committed dc261 plus
the ten Rust DefUse postimages. This is readback of that actual receipt, not
a new execution by this task. C/Kotlin/TypeScript and other DefUse entries
above are static source findings; no runtime failure is inferred for them.

Rust/Go supplier hardcut is assigned to T1 by the incoming coordinator.
Assignment, an active exact-path claim, source completion and owner execution
are separate observations; the inventory JSON records actual current claims.
No consumer implementation or shared API change is authored by this update.

Acceptance covers both real from-tree HIR/semantic-bundle calculations and
actual extraction walks, preserving existing outputs, quarantine, ordering,
source/function binding and original current refusal. Reuse public scoped
read APIs. No raw getter shim, uncontrolled reparse, controlled-input
ParseResult fallback or error-to-None conversion is allowed. Compile and
execute the affected owner/feature combinations with nonzero tests and
existing authority/work/memory/cancel oracles before closing these owners.

`supplier-r45-source-inventory.json` records committed path hashes, entrypoints,
ownership and scoped acceptance; `supplier-r45-peer-owner-archive/manifest.json`
binds archived receipt/result/summary hashes. R38 common/Java, R44 445 and
cursor source absorption do not qualify these remaining supplier owners.
