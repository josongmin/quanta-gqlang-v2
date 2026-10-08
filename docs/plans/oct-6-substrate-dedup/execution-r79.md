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
- Later coowner commits `6e77122b888` (canonical callsite reads/equality) and
  `109fd7575d2` (Python exception proof) are observed separately. The new Native
  crate still has zero committed files at `109fd7575d2`; its compatible supplier
  and consumer packet remains WIP. R87 execution retains its original `6bab`
  snapshot rather than borrowing these later commits' identity.
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
- R86 rejects a changing `quanta-index` overlay before issuing an owner input.
  The successor R87 selects the normal committed-HEAD policy for every discovered
  sibling: QGLang `0bde21d855b17f9c22bee6f1bb679195b4dbff0f` and Index
  `703d0e6899599ab4ab69c051089a9e4eda497ab0`, both with empty overlays.
  Semantica remains a complete guarded WIP capture at `6bab127740a3`.
  This different input excludes sibling WIP; it is not its qualification.
- Evidence root: `/Users/songmin/.codex/task-evidence/oct8-closeout-execution-r79/`.
  The normal lane is `gqlang-closeout-r79-snapshot-artifact`, one heavy owner at a
  time in this execution queue. Source drift rejects a mixed input before tests.

### QGLang repository gate

`just ci` passes on clean committed `67377d561da8428b8abec7214ef5e3da46922d46`
in a managed verification worktree. The local CI source digest is
`c57fce68aa7b5df516bae61178fc8f7f706895515aa263b027e541e84b3528c4`.
Doctor, offline external-consumer preflight, static, test and policy all pass.
Nextest discovers/selects/executes/passes 1,514 cases with zero failures,
ignored cases or leaks. Current Clippy and MSRV Clippy are included in static;
the size gate checks 283 Rust files. The external facade executes its one
positive program and all 23 negative cases, with 156/156 pinned public API roots.

The earlier mutable-main attempt exits2 because the document source changes
during static; its partial leaves are not a passing repository gate. The clean
worktree successor exits0, validates the current-run Nextest stream and is then
archived. Its log, inventory, events and hashes are retained outside the worktree
in `qglang-ci-67377d5-receipt.json`. This is QGLang local CI acceptance at that
HEAD, not downstream Semantica or installed product acceptance.

## Original Shape/SCC objective on main

Read-only caller tracing at Semantica `3b8f86dd9ea` confirms the original
deduplication and ownership change is committed. Runtime heap analysis reaches
`ShapeAnalyzerPort` and the Shape adapter's PTS/component classification; its
graph facade reexports `codegraph-graph-scc`'s bounded iterative Tarjan owner.
PTA Andersen, the PTA SCC adapter and abstract worklist also call that core.
These implementation paths have no live diff against main. The separate dirty
Shape port refusal projection belongs to an existing runtime-custody claim.

Thus no additional Shape/SCC traversal implementation is assigned. Pending
owner acceptance, Native/Original publication and product qualification below
are separate completion obligations; they do not mean the committed dedup
source is missing. Source tracing alone supplies no runtime verdict.

## Actual parallel work and integration

| Lane | Executed change or result | State |
| --- | --- | --- |
| A / parser | Explicit hierarchy paths for `correspondence_v3`, `edges_v3`, `implicit_object_v3`; no-Java imports scoped to their existing test/Java features | Integrated `d29ac508b4d`; parser owner acceptance pending |
| A / native API | Core IR Foundation imports `codegraph_native_allocation_core::copy_string_v1` through its existing public root export | Authored WIP; do not integrate without the native supplier/manifests/caller packet |
| A / CFG reader | Explicit index and error signatures for the existing `IntervalReaderV3` callbacks; no algorithm or paid-storage change | Authored after reached R81 `E0282`; leaf format/diff PASS, successor owner pending |
| A / paid row type | Public opaque nominal `PaidBoolRowsV3<T>` for the sealed storage trait's associated type | Authored after R82 `E0446`; private fields and domain-only constructors/loan methods retained; leaf format/diff PASS |
| A+B / facts compile | Canonical FileId error import, two value-loan lifetimes and receiver checkpoint's typed unpublished-refusal mapping | Authored after R83; leaf formatting/whitespace checks PASS; general producer/output proof still open |
| A+B / core DFG compile | Existing sibling read import, common internal reader visibility and canonical `UnsupportedOperation` refusal | Authored after R85; companion Source/Dynamic rejection test corrected with zero-birth and cancellation assertions preserved; successor owner proof required |
| Coowner / core DFG nominal | `OriginalDfgIdPathV3` becomes an opaque public nominal, retaining private fields and internal methods | Live successor of R87 `E0446`; B observes the existing exact claim/source repair and adds no competing edit |
| A+B / contract types | Constraint sibling comparator visibility, current five-argument callback completion, and three precisely typed cursor callbacks | Authored after parser-no-Java R87's nine diagnostics; leaf format and affected boundary policy PASS; successor compile required |
| A / test relationship | Exact synthetic retrieval wire and Source path assertions precede the existing positive Arc-demand checks | Scoped Honesty16/16 and exact Foundation owner1 pass; the source belongs to the coupled Native WIP packet |
| A+I / feature policy | Remove redundant Python-search floor root, reuse the exact three-root isolation class, retain parser activation guard and synchronize its registry/inventory | Seven-file HEAD-only candidate verified and integrated on local Semantica main as `3b8f86dd9ea`; foreign manifest bytes preserved |
| B / interproc hierarchy | Five explicit child paths under the existing `delta_result_carrier_v1` owner | Authored WIP; physical paths, leaf rustfmt and scoped diff checks PASS; no successor owner compile inferred |
| I / interproc controls | Tie retained metadata owner and iterator callback loans to their existing lifetimes; checked `usize`→`u64` work conversion records first overflow before the visitor | Three-file narrow normal claim, compatible incidence-owner reconciliation; leaf rustfmt/scoped diff PASS after one local formatting correction; coupled owner compile pending |
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
| R86 / no issued owner input | Source capture rejects, exit1; compiler not started, zero tests | Index overlay changes after capture; successor uses normal committed sibling inputs, preserving exact validation |
| R87 F2 / `70945899e8f1ef1ea343d138a38646aff84cc7488631e326e2bedc38d8865bc5` | Finished RED, exit101, zero tests | Earlier repairs pass; Core DFG reaches one restricted nominal associated-type leak (`E0446`); existing coowner repairs the live successor |
| R87 parser no-Java / same source digest | Finished RED, exit101, zero tests | Independent owner reaches contract-types: comparator scope, stale three-argument callback completion and three callback loan generalities (nine diagnostics); A/B repairs authored |
| R88 Foundation / `7c072270d830bff173ad5e07291f281d36275178795e52458aac3db5258fefe9` | Published GREEN, exit0, exact required test1 | Fixed-wire and Source semantic assertions execute with the existing positive Arc-demand checks |
| R88 parser no-Java and F2 / same source digest | Finished RED, exit101, zero tests each | The previous contract-types and DFG nominal compile blocks pass; both reach three filesystem descriptor module mounts (`E0583`). Their existing exact coowner adds explicit parent/child paths in the live successor |
| R89 parser no-Java / `3e7bccb294d9a229d49883c000938667e2506bd7c9a9f7d3cde1b3a632b763ca` | Finished RED, exit101, zero tests | Descriptor mounts pass; SourceWire reaches four import/unused/projection-Clone diagnostics |
| R89 F2 / same source digest | Finished RED, exit101, zero tests | Its broader feature closure reaches18 SourceWire diagnostics: module/constant/native-owner imports, one missing local, callback bounds, shared-payload generic error mappings and work-charge type, plus overlapping parser diagnostics |
| R90 parser / `e67525feeae7bddd535ba40468aea699a2ced0f3ff3f825cb4b2cd79be184d1f` | Finished RED, exit101, zero tests | Earlier SourceWire type/path causes pass; two private-module constants have unreachable public visibility |
| R90 F2 / same R90 input | Finished RED, exit101, zero tests | Same two visibility errors plus one unused child-module JSON document import; current coowner already removes that import |
| R91 parser/F2 / `cf8747e9793cd1912543a479c036199495de8d4c3acf7b86087e4d6961cfef60` | Finished RED, exit101, zero tests each | SourceWire compiles; ports `BootstrapInputMutationV1` derives Eq/PartialEq over a retained candidate that deliberately has no uncontrolled content equality |
| R92 parser / `25647d56744a6563f5fb39eb6b4173c26e1e8dd0aba568c59d3838599701988a` | Finished RED, exit101, zero tests | Ports compiles; two error-shared matches omit `OriginalGenerationSourceInputRefusedV3` and `SourceAuthorityRefusedV3` (`E0004`) |
| R92 F2 / same source digest | Finished RED, exit101, zero tests | Broader closure reaches91 interproc-contract diagnostics; opaque result consumer migration51, imports10, incidence API/derives9, visibility8, module mounts5, replay derives3, factory APIs2, overlay/private storage1, unsized generic1 and callback lifetime1 |
| R93 error-shared / `a41f3dd16d675c65f7046180d00f08d52cabdbdb5d094aa2e663f0d7ad3d0761` | Published GREEN, exit0, exact regression1;34 filtered | Eight independent finite cause/category/phase cases pass; not the whole library or later parser |
| R95 parser bootstrap / no issued owner input | Operational failure, exit1; compiler not started | Dynamic dataclass module missing from `sys.modules`; retained separately, fixed in fresh R96 bootstrap |
| R96 parser / `5568d127379b48bf489487fcfaf1cda600c519ee7242857409cdaeab0f889c2c` | Finished RED, exit101, zero tests | Error-shared passes; Kernel contract reaches18 errors (15 coded, three denied imports) in its current producer/consumer cut; later leaf repairs have separate postimages |
| R97 parser / `9c838b5e83abd78f6e05d523b23079fe576a233a3a60fa28511adcaa000a16bc` | Finished RED, exit101, zero tests | All four authored Kernel postimages captured; their four R96 diagnostics disappear. Remaining14 belong to this input; later three diagnostic files change under the existing minimal-import owner |

Each F2 run has its own `f2-native-owner-rNN.json`, `owned-proof-results-rNN.json`
and archived stdout/stderr. No required F2 witness ran. These are compile failures,
not registry, Preview or deadline behavior failures. R79–R85 stop the sequence
after the failed first owner. R87 also runs independently owned parser and SCC
requests on the same leased input, preserving each scope's result. The parser
no-Java request is
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

### Source-bound policy checks

The narrow Honesty regression first rejects the fixed-backing test's generated
retrieval value because it is never asserted. The additive semantic identity
assertions preserve all existing Arc-demand checks; Honesty then passes16/16,
exit0, with 70 selected path/config hashes stable before/after
(`6b18b5403ee52277e0489174eff6ebc720c9b06c2e76188eb6265594cb34f319`).
The exact reachable Foundation test request is
`547b318cdbdcb2d856aaeb46fdb26728`, lib/exact/package-all-features.
R88 executes and passes its one required test, with zero failed/ignored cases.
`owner-postflight-r88.json` independently validates the normal lease/custody,
exact projection, source digest, witness, finished authority and immutable
publication marker. Its SHA256 is
`4891c87427e9ba3e0aa9b79c4e2a37f68e490ba2abc3ae457352f6db6df172f6`.
The two RED compiler receipts have no publication-complete marker; they are
archived as failed diagnostics and are not accepted owner scopes.

Stable scoped checks also pass six impacted build-phase boundary policies,
feature coverage and the one registered serde edge (80 selected inputs), and
GOD checks for24 Native/core/facade plus13 Shape/SCC files (46 selected inputs).
The two contract-types source fixes pass seven affected build-phase boundary
policies. These are source/static checks, not compiler or product acceptance.
`supplier-policy-{gates,god,honesty,selected-inputs,claim}-r87.json` retains the
exact arguments, selected manifests and claim record as a session-output
transcription, explicitly not a QBC receipt.

The language-substrate literal/build backstop passes; its authoritative
Cargo-tree gate/test phase is not run. The root-test topology command selects
no governed files here and supplies no proof. Required-feature-floor vacuity
initially exits2 for the SDK Python search test on both WIP and committed HEAD.
The redundant direct floor root is repaired as a separate complete policy bundle:
`search-runtime` already reaches `parser-python` and the floor transitively.
The before/after local feature closure is exactly83/83; the existing registered
three-root class remains equal to Cargo's required features. Both integration
and subtest dispatch links are retained. The new guard rejects removal of the
transitive parser edge instead of requiring a redundant root.

The seven-file isolated HEAD candidate passes floor-vacuity33 and guard6 tests.
Live selected casectl tests pass2; the isolated archive passes1, while its
metadata-dependent test is NOT_QUALIFIED because archived sibling topology is
absent. Static target/link resolution passes2 with zero unresolved links.
No search Rust execution or feature fan-in reduction is claimed.
Normal shared-main source-commit integration creates `3b8f86dd9ea7f94a2426277311037ea9d6480a6a`;
exact-index Ruff, staged integrity and safety hooks pass. All seven live
postimages remain byte-identical, including excluded foreign runtime feature
and dev-dependency hunks. `policy-integration-postflight-r88.json` retains the
append-only commit attestation and preservation hashes. Whole-repository
Semantica policy acceptance remains incomplete.

## Independently accepted SCC scopes

On the same R87 frozen source digest, core CFG30, concurrency domain6 and
concurrency engine38 pass with zero failures/ignored cases. The postflight
verifies all74 required prior witness names, distinct run IDs, exact owner
projections and immutable publication marker/receipt hashes while holding the
normal snapshot lease. `c-independent-postflight-r87.json` has SHA256
`20affed23adf428087a88f5007c1e15aeee2f0cc3933c8cf27f71e72b000d940`;
`c-independent-r87-archive/manifest.json` preserves their logs and receipts.
These three scopes are GREEN at this input; this is not product qualification.

Import graph32 also runs independently and passes on the same R87 input. Its
receipt SHA256 is `2cfa67568530f47dbe2e6a0a6f3ab3aa4d87fcb74f51a7c7e6a389970f0c685e`;
`c-consumer-run-r87-import-graph/state.json` verifies all32 prior witnesses,
minimum count, exact owner projection and immutable publication. Four accepted
SCC scopes total106 executed tests, with no failed/ignored cases.

The later continuity scan to live `109fd7575d2` finds matching selected compile
inputs for core CFG/domain/engine, with only a transitive Foundation cfg(test)
file changed. This is scoped source continuity, not a new GREEN run. Import
graph's closure changes at the language-plugin output-admission boundary, so its
R87 success is not transferred to current WIP. Metrics14 and the other seven
queued scopes still need their own successor owner results; unrun targets stay
NOT_RUN rather than RED.

The changed import-graph dependency/source gets a new R89 run and passes32,
with all32 prior required names, zero failed/ignored cases and immutable
publication validated. `c-consumer-run-r89-import-graph/state.json` preserves
its exact queue/source/witness projection; the new receipt SHA256 is
`637498442b7e1555c724454b39fab16db9226fdd360a76cbda3f9f9dfbcb49fc`.
This supersedes import-graph's older input for that scope without combining
unexecuted metrics/rule/runtime owners into its verdict.

### R89 successor input

After policy integration, R89 captures Semantica `3b8f86dd9ea` plus its complete
guarded WIP and committed siblings QGLang `ec196f60e8b0` / Index `9a47d86a5cd0`.
The topology identity is
`3b8f86dd9ea7f94a2426277311037ea9d6480a6a+ovl-303ae30a997cb629c3c0ec9025c4f50e+ext-c3fd5f77c68e6c60d09a`.
Its parser/F2 runs and postflight retain a normal lease. Results are not accepted
until the finished source-bound receipt, publication and required witnesses agree.

Parser no-Java finishes RED101 with zero tests after passing the descriptor
mounts. The reached SourceWire diagnostics are an obsolete capacity import,
two unused reexports/imports and a projection's ordinary Clone derive over a
deliberately non-Clone generation authority. Current source must be reconciled
with existing writers; adding ordinary Clone to authority is not a repair.
The 12 consumer requests are re-resolved on this exact frozen input and preserve
all request IDs, complete projections, minima and required witness names.
`c-frozen-consumer-queue-r89.json` binds its HEAD, topology and every sibling SHA.
Resolve success remains distinct from execution.

Both R89 owners finish RED with zero tests. `owner-postflight-r89.json` has
SHA256 `2d543539a17577925c42f98a42108bc78dc70d441fdf940ecac4989534433fb5`;
the failed receipts/output pairs are archived without a GREEN publication
claim. The broader F2 input reaches18 diagnostics, not the parser's four.

Current coowners remove the obsolete import, unused classifier import and
projection Clone derive. B scopes the remaining text-copy reexport to
`original-repository-fd-capture-v3`, matching its only caller; leaf format and
diff checks pass. The authority remains non-Clone. Its owning postimage is
`ed7cba0afb3d661d9b2bc97b6cb5964ea7d133f268227f23cba5f57af2d646d3`.
These authored fixes require a new input and compiler result.

Feature-aware locked/offline trees on this frozen input show import graph
avoids SourceWire, while metrics/rule/current-target/navigation/community/F7
all reach it. `c-tree-r89.json` binds exact argv, selected package manifests,
root/lock and output hashes. It routes the next execution rather than treating
dependency reachability as a passing or failed test.

The remaining broader F2 diagnostics are also absent from inspected current
postimages: explicit header mount/mapped births, canonical native dependency,
fixed receipt size and magic-constant paths, adapter callback loan bounds,
explicit pin/shared-payload error transport and checked usize-to-u64 work charge.
The header child's use of `Cause` makes its parent import necessary; its old
unused diagnostic follows the missing module and must not trigger deletion.
A/B preserve existing owning edits rather than add duplicate fixes.
The manifest subtree's unrelated existing rustfmt differences remain; they are
not hidden by the leaf checks or formatted over foreign WIP.

R90 captures those compatible current postimages for parser/F2 successor proof.
Its own source identity/results stay separate from R89's failed diagnostics.
No full Native or Original source packet is integrated from this readback.

R90's two constants are private implementation data: their only consumers are
local auxiliary-policy functions, which the parent reexports. B removes only
`pub`, retaining values/order and passing leaf format; postimage SHA256
`48b3c8debd29682f81ebc4e7ebb0671a6d696135c5927b2ff229cee1c183319d`.
R90/R91 postflight hashes are respectively
`554257c03e4ed0cd73ede4bc40b5b53448e468a7bf7f5b05cb87c018049f87fe` and
`714d8284c9e403b071b24701212a4e7dbeaa527ec524cabd98c32851ff8f3bce`;
both preserve the failed receipts without accepting zero-test compiles.

R91's wrapper equality cannot be repaired by adding an ordinary Eq implementation
to retained content. Candidate values expose an ordinary-only fallible comparison;
Original content requires an authenticated read. B removes only wrapper Eq and
PartialEq, retaining Debug, the existing fallible clone path and outcome/receipt
equality. No workspace equality consumer is found. Its postimage SHA256 is
`fa7d10e074c9b24114cdf2a5f179d414c37ae228c4e7ec51a0ce87e03dd32092`;
leaf format/diff pass. This public trait-availability change remains inside the
unintegrated compatible Native packet and needs its consumer/compatibility review.
R92's parser passes that ports boundary and reaches the two exhaustive error
projections above. The existing `OCT8-ERROR-SHARED-R115-SOURCE-REFUSAL-MAPPING`
owner has already added both arms in current WIP. The successor preserves the
finite nested cause and phase rather than projecting every refusal to InternalBug.
Its regression contains eight independent cases: cancellation, deadline, memory,
allocation, missing head, timed-out I/O, stale generation and copied authority.
This authored mapping is not inferred to pass from its source presence. R92 F2
also ends before tests, on the broader interproc-contract boundary above. Its
postflight SHA256 is `d3f7fa98953d4035fbcdf04a8784ffa5d3d8d51ff56e5275fe503045c0ccd7a4`;
both failed receipts/logs are archived under lease without publication acceptance.
After-run disk cleanup completes normally; no artifact admission floor is lowered.

Before the bounded successor repairs, readback of all22 diagnostic-bearing
interproc files matched R92's frozen bytes.
The51 opaque-result diagnostics include cascades from direct fields, result
literals and Clone/Eq assumptions. They are a producer/consumer migration, not
51 isolated bugs. Canonical ordinary projections already return typed storage
errors; retained data requires authenticated controls. Public heap/domain and
context-sensitive result carriers plus bare tuple/set/vector/Option APIs and
their real taint consumers must change together. Returning empty/None on missing
authority would fabricate a successful analysis. Explicit module mounts and
private signature/import repairs can proceed independently, but do not close
this cross-crate contract cut. Exact cause/path/claim data are retained in
`r92-f2-live-reconciliation-r93.json`; source-only Native/Original readiness uses
44 stable selected files in `supplier-final-readiness-r93.json`.

The bounded source successors retain their own postimages: five hierarchy mounts
in `interproc-carrier-mount-r92-b.json` and three control signatures/work count
in `root-interproc-control-final-r93.json`. Their formatting/whitespace checks
pass. The R93 capture predates these edits, so its error-projection proof cannot
qualify them. Two factory-method errors also originate from the missing ordinary
child mount; no new fallback factory is introduced. A new compiler diagnostic
count must be executed rather than inferred by subtracting edited lines from91.

The ordinary PointsToResult cut exceeds the initial nine-file assignment.
`ContextSensitivePtaSolutionV2::from_legacy_projection_v1` has17 caller files
across PTA adapters, runtime, runtime-analysis and tests. The real facts caller
`adapter_solver_facts_entry.rs` returned `()` through its append helper at this
R93 readback, and the access-path API has about30 taint-solver consumers. The
canonical storage owner `OCT8-POINTS-TO-RETAINED-STORAGE-72` covers the producer;
no active claim then covered the complete consumer migration. A retains the source rather than
installing partial projections that return empty data for missing authority.
The exact expanded path/claim inventory is
`r92-f2-points-to-consumer-cut-r93.json`.

### R93 exact finite error projection acceptance

Canonical owner request `21efcd49eb4af2faa7cd856d7622bc8d` passes the exact
`failure_category_ext::tests::nested_original_and_source_authority_refusals_preserve_finite_cause_v3`
libtest: one passed, zero failed/ignored,34 filtered, with all eight cause/category/
phase expectations checked. Its guarded input is Semantica `3b8f86dd9ea` WIP plus
committed siblings, campaign source digest
`a41f3dd16d675c65f7046180d00f08d52cabdbdb5d094aa2e663f0d7ad3d0761`.
The finished receipt SHA256 is
`143e1661ca06cd111f38789b1c5ae5817be8d99e7af9f4f9cc366c2024846200`;
the immutable publication marker binds it. Postflight runs under the held source
lease and archives the exact witness, receipt, stdout/stderr and marker; report
SHA256 `021b139ad4b4718a7002f86d3de6fb921287137d8a08b9e4c9e54ab63f6dfcca`.
Composite topology identity and hashed campaign source identity are validated
separately, rather than comparing these different representations as strings.

This accepts that finite projection on its own input. It does not qualify later
interproc edits, the entire error library, the latest parser, SDK or Original
success. R94's full parser/F2 replay is not run against the known unfinished F2
consumer cut. R95's parser-only bootstrap stops before owner resolution or
compiler execution: its dynamic Python dataclass module was not registered in
`sys.modules`. `parser-r95-operational-failure.json` retains this operational
failure separately from Rust RED. R96 corrects the module import, verifies it
with configured Python3.12 and captures a new guarded parser-only input; no
failed R95 input or result is rewritten into a passing receipt.

### R96 parser successor and current consumer reconciliation

The parser-noJava whole-library `check` finishes RED101 on campaign source
`5568d127379b48bf489487fcfaf1cda600c519ee7242857409cdaeab0f889c2c`:
Semantica `3b8f86dd9ea7` with guarded WIP and all discovered sibling committed
HEADs, including QGLang `b1957acef70b`, with empty sibling overlays. It passes
the earlier error-shared projection boundary and reaches18 errors in
`quanta-runtime-kernel-contract`:15 coded diagnostics and three denied unused
imports, across13 files. No tests execute. The finished receipt SHA256 is
`2e975d44888d93a5bb9319cd18875b1979bf4a20af0a1ea2dc93ad637ad2f14f`;
the leased custody/postflight archive report SHA256 is
`5991dc76efd065f3b7de573d1e4fe9a102f777114112a01488425d6f812170cb`.
RED has no passing publication or reuse authority.

`r96-parser-full-reconciliation.json` retains the exact diagnostics and current
claim/path hashes. Reached causes are feature-scoped source authority imports,
source-text/context consumer APIs, generic callback error propagation, finite
identity-material/observer errors, output trait bounds, an empty feature-off
storage match and a moved envelope. These are the reached Kernel closure;
R92's91 F2 diagnostics remain a different historical input, not the current
residual count. The owned envelope successor changes only the private helper
and sole caller to borrow `VerifiedCanonicalOutputEnvelopeV1`. Decoder ownership,
admission and rejection remain unchanged; exact reverse reconstruction,
scoped formatting and whitespace checks pass. Its postimage is
`6c76d4f9a67278cc03f77777341c5f3ab562d4dac5ff5b7691048947da40122b`.
`r96-kernel-envelope-borrow-a.json` records the reconciled and released claim.
This later source repair is WIP and has no successor Rust compile acceptance.

Two additional bounded Kernel repairs retain separate postimages. The exhaustive
committed/pinned identity wrapper now returns the same existing
`CanonicalIdentityMaterialErrorV3` as both SourceWire producers; its forwarding
body is byte-unchanged. The two observer consumers now propagate issuance refusal
as `SourceIssuanceControlRefusedV3` before payload/row visitation, with `E::from`
at the generic resource boundary. Callback and first-refusal order are unchanged.
Their exact patches/claims and leaf checks are in
`r96-kernel-identity-material-error-a.json` and
`r96-kernel-observer-issuance-consumers-b.json`; all narrow claims are released.
The independent four-file static readback is
`r96-kernel-leaf-independent-review.json` (SHA256
`29a0b62f856ccb6dd717607e1c1720366a68d6df7754de04ab366f1552f009f8`).
R97 captures their later complete guarded input for a parser-only successor;
neither R96's failed receipt nor these leaf checks establish its result.

R97 then finishes RED101 on campaign source
`9c838b5e83abd78f6e05d523b23079fe576a233a3a60fa28511adcaa000a16bc`,
with the same declared committed sibling heads and a fresh Semantica overlay.
The four authored postimages are present; none of their four R96 diagnostics
is reported. The actual remaining Kernel count is14 (11 coded errors and three
denied imports), zero tests. Finished receipt SHA256
`d9c0106d8a01c8a332ff3319c00bc34077ca4bee937e1600d5efdc8fff24c16a`;
leased postflight/archive report SHA256
`ae89ba9412d3917df38da23741dd4b267881738735c4ffdd87126bac3d04e8e9`.
This confirms diagnostic progression, not a passing owner or product result.

`r97-parser-full-reconciliation.json` (SHA256
`e331458209baa5f7664e5527c99e613abad36749caf9deb9efe5a70d2b0fe3a0`)
records9 remaining diagnostic files. Six still match the frozen input at readback;
three have later postimages under `OCT8-KERNEL-R127-MINIMAL-IMPORT-CLOSURE-ROOT`
alongside their existing owners. The14 count is therefore not claimed as the
latest live-main residual. Complete that existing feature/import cut and the
remaining context/text, generic callback error, output bound and feature-off
storage consumers before a new broad acceptance capture. Keep both source
identities and the unfinished Native/Original implementation boundaries separate.

The parallel source-only readback in
`r92-nonopaque-current-reconciliation-r96.json` finds11 of13 non-opaque
interproc files changed since R92. Pair exports/journal, PtaError import,
state derives and factory repair are under
`OCT8-PTA-PORTS-DELTA-INCIDENCE-CONSUMER-MIGRATION`; retained overlay and unsized
retraction remain in its coordinated boundary. KCFA replay receipt traits also
reach runtime's `frontier_v1.clone()` and require fallible ordinary or authenticated
retained copy with typed admission failure. No independent derive-only repair is
assigned, and the old count is not reduced arithmetically.

`r96-opaque-points-to-consumer-current-partition.json` freezes13 selected path
hashes and current claims. V1/V2 copy and facts/taint append migrations now have
active child owners; the R93 absence of a complete consumer claim does not mean
these later sub-boundaries are unowned. V2's public legacy conversion is now
fallible in WIP and still requires all17 distributed caller files to change
compatibly. Facts projection/append must preserve storage and digest causes,
avoid partial facts and retain the original first error. Alias/access-path/
indirect-call APIs and real consumers must distinguish `Ok(None)` from refused
storage, with deterministic output and no empty/default success on refusal.
The artifact is moving-WIP source evidence, not compile or product acceptance.

Next capture requires the complete compatible Kernel/source-authority and
interproc producer/consumer postimages. Resolve parser no-Java/Java, F2 and
actual runtime/SDK owners on that declared tuple before integrating the coupled
Native packet. Existing claims and foreign manifests/locks are preserved;
neither a partial child file nor a typed refusal is removed to manufacture GREEN.

## Current Original source boundary

The [proof handoff](../../work/substrate-repair-checkpoints-2026-10-07.md)
keeps independent historical and coowner source observations separate.
The B-lane readback at `d29ac508`, refreshed against `6bab127740a3` and its
current overlay, distinguishes these authored parts:

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

The latest source readback also confirms that runtime and SDK use the current
four-argument native workspace API; the earlier external `members_v3` mismatch
does not apply. The generic `FactExtractionRequestV1` and
`DefUseFactsParseContextV1` still carry only `current_control_v1`, and their
extractor returns ordinary `DefUseFactsV1`. General admission/keeper propagation
therefore requires a coupled producer/consumer API change. Existing first-facts,
retained-syntax, facts-storage, source-world, Java-callable and PTA transaction
claims cover those seams; this queue does not install a second authority issuer
or treat refusal deletion as a success implementation.

The final selected-source readback at `3b8f86dd9ea` confirms all three general
success seams remain open and reachable. Runtime
`cfg_dfg_authority_issuance.rs:1012` reaches the DFG source-world guard, and the
controlled file batch reaches Tree C's loan-time refusal. The real Java facts
frontdoor rejects controlled input before its ordinary result constructor;
the generated-field keeper is a specialized private candidate. PTA workspace
generation reaches its nonempty relation guard and final owner materializer;
the general transaction still refuses, while its success entry is void-return
only with zero definitions/uses/callsites. Stable selected hashes and existing
first-facts/DFG/PTA owner claims do not turn these into success implementations.
Closing this scope requires actual admitted producer-to-final-output APIs and
their regressions, beyond the already committed Shape/SCC dedup source.

## Remaining execution and completion criteria

1. **FCP-COMPILE / P0:** repair only newly reached owning compile causes; capture
   the complete supplier/caller tuple, pass parser no-Java/Java and actual
   runtime/SDK owners, then integrate the compatible source packet normally.
2. **CHECKED-RETAINED-CONTROL / P0:** finish the three success boundaries above;
   prove same-operation loans, admission before growth, first refusal, cancellation
   and final-drop refund through actual producers.
3. **Independent owner proof / P1:** execute Shape/Cost, PTA/taint/DFG, the five
   pending SCC consumers (rule, metrics, current-target, navigation and community),
   SDK2/community2 and F2/F7 after each complete closure is
   available. The C request map (`c-consumer-owner-request-map-r79.json`) is not
   a result. `c-scc-continuity-d29-r83.json` verifies the eight earlier receipts'
   publication hashes: dominance3 and worklist77 match pinned committed `d29`,
   with no external path dependencies. Rule34, concurrency engine38/domain6,
   metrics14, import graph32 and core CFG30 have changed IR-build dependency
   closure at that older checkpoint. R87 now supplies accepted successors for
   core CFG30, concurrency domain6/engine38 and import graph32; rule34/metrics14
   remain open. Import graph's later admission dependency change is reaccepted
   by R89's32 executed witnesses; that successor retains its own input rather
   than updating R87's identity. This older continuity
   does not qualify moving WIP or later `6bab`. Existing accepted core/facade and scoped
   D4/D5 are not reassigned as source repairs.
4. **Policies and product:** run affected mandatory gates and receipt-bound real
   Graph/CHECK process oracles. Positive installed CHECK requires release-owned
   signed x86_64 Linux profile/immutable slots/independent approval/key/generation;
   this Darwin host has no installed positive result. FCP-INSTALL does not block
   independent local compile/algorithm proof. Reuse evidence for product rollup.

No ticket closes from static readback, request resolution, a failed compile or
mixed-source counts. Hosted CI, push and installed release publication are not
performed. Full closeout remains incomplete.
