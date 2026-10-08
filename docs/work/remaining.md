# Remaining work

Updated: 2026-10-08. Initial committed-source audit input:
Semantica `871224b2059422a8f36eca91ffb0c9119238ac9c`, with QGLang audit input
`01cfe39ca0571b4fc099d41b24628fda70e754fe`; foreign dirty overlays are
excluded from that audit. R49 source integration is listed below; source
integration and runtime qualification remain separate.
Earlier dirty-overlay attempts retain separate issued overlays on Semantica
`af563ffa29bd`, `35183fbdba80` and `73f2483f800e`; the
[R53–R60 handoff](substrate-repair-checkpoints-2026-10-07.md#r53r60-reached-repairs-and-successor-input)
records actual blockers and accepted scopes; the
[R60–R63 candidate](substrate-repair-checkpoints-2026-10-07.md#r60r63-source-stability-and-clean-candidate)
isolated rule/community from moving co-owned source. R63 rule execution is
accepted34/34 and integrated as `a23ced26e09`; community stopped before tests at56
committed parser compile errors. R64 JSON projection13/13 is accepted; its other
independent D5 solver225/225 and API-absence2/2 proofs are accepted. Shape/Cost
passed20/20 each. R65 accepted dominance3, concurrency engine38/domain6,
metrics14 and core-CFG call graph30. R66 import graph32/32 is accepted; actual
taint-Zipper32/32 is accepted on the same issued source.
The parser enum-export correction is authored and reviewed, not execution-qualified or independently integrated.
GQLang `just ci` passed locally; this does not qualify Semantica runtime or installed product.
The [R67–R71 integration audit](substrate-repair-checkpoints-2026-10-07.md#r67r71-main-integration-and-missing-test-repair)
confirms the three earlier source checkpoints remain on main. It also found and
fixed a Rust regression hidden inside a raw string fixture: actual77/77 owner
acceptance and one-file integration `f574fa82bf1`. Existing supplier/facts work
is not implemented a second time. R70/R71 accepts385 cases across the five
ordinary language libraries and exact SCV2 module on one issued source matching
main `f574fa82bf1` at active dependencies. The
[R72/R73 follow-up](substrate-repair-checkpoints-2026-10-07.md#r72r73-d5-closeout-and-independent-main-diagnostics)
accepts D5 solver225/API2 on that same source and closes D4/D5-CUSTODY. The eight
requests total612 cases; the separate R72 worklist77 is not added to that total.
SDK2/community2 remain unintegrated; these scopes do not close original custody
or whole product.
The [R74 remaining audit](../plans/oct-6-substrate-dedup/readme.md#current-remaining-audit--r74)
maps all18 active IDs into six execution groups. Current dirty runtime→PTA code
now calls the DFG original issuer, correcting the earlier missing-caller claim.
Remaining source work is its changed member-argument handoff, admitted workspace
transactions/publication and the general facts output-admission seam, alongside
complete supplier/module integration. Source presence has no execution verdict.
Original consolidation provenance is retained in ADR-0001/ADR-0005.

Only unfinished work belongs here. Integrated architecture is recorded in
[ADR-0004](../adr/0004-sealed-owner-facts-and-bounded-projection.md),
[ADR-0005](../adr/0005-substrate-reuse-and-consumer-projection.md), and
[ADR-0003](../adr/0003-frontend-downstream-integration-boundary.md).
Source presence, focused proof and same-source product qualification are distinct.

Current frozen execution inputs, witness selectors and receipt locations are
in the [active proof handoff](substrate-repair-checkpoints-2026-10-07.md).
Its current candidate and historical receipts retain their own source tuples;
failed zero-case compilations do not become execution proof.
The [current execution plan](../plans/oct-6-substrate-dedup/readme.md) contains
non-overlapping write sets, before/after changes, tests and serial main checkpoints.
The [ticket index](../plans/oct-6-substrate-dedup/tickets.md) maps all 18 active
closeout IDs to priorities, code/proof boundaries, owners, dependencies and DoD.
The proposed A-series below is a separate roadmap, not an implementation defect
or a prerequisite for this campaign. This file owns current ticket states.
The [parallel dispatch](../plans/oct-6-substrate-dedup/parallel-dispatch.md)
owns write-set allocation and checkpoint handoffs, without creating a second queue.
Completed repair decisions belong to
[ADR-0005](../adr/0005-substrate-reuse-and-consumer-projection.md#subsequent-integrated-repair-decisions).

QG-DIAG is resolved by the existing exact-field consumer contract; its durable
decision is [ADR-0008](../adr/0008-private-diagnostics-primary-location-contract.md).
The private wire is unchanged. Runtime execution remains a separate claim.
D4/D5-CUSTODY is scoped accepted: SCV2 34 and solver225/API2 have source-matched
finished receipts and publication on the successor input. Its
[ticket](../plans/oct-6-substrate-dedup/tickets.md#d4d5-custody) retains DoD and
evidence. Later affected-source revalidation is a common gate, not unfinished
D4/D5 implementation.

## V3 downstream product proof

Owner: Semantica SDK/runtime/release. Producer maintenance is integrated.
Existing fixtures and lane registration are integrated; execute their oracles
before claiming completion. Broader AG5/IRC releases remain downstream-owned.

| ID / state | Required work | Acceptance |
| --- | --- | --- |
| FCP-COMPILE / BLOCKED_COMPILE | Current clean committed f574 parser all-features lib check reaches105 compiler errors/exit101,0 tests on R72's fixed sibling tuple. Its publication is incomplete: this is an archived pretest diagnostic, not an accepted qualification. Supplier/module postimages remain co-owned WIP. The committed Java DFG consumer also lacks its complete committed Foundation shared-backing supplier. Freeze and validate the complete tree-sitter→Kernel→Foundation→SourceWire/parser bundle and actual SDK/PTA/runtime-analysis owners before integration. Earlier R49–R63 failures keep their original inputs. | Canonical product build and SDK owner rails on one source tuple; actual current diagnostics and finished publication, not old error counts or partial source imports. |
| FCP-PTA / REVERIFY | Verify the seed-consumption lifetime repair against the historical first `PtaOccurrenceAuthorityRefused` boundary; no current reproduction is claimed. | Current owner/bootstrap succeeds, or a reached refusal identifies the exact remaining owner repair. Source changes alone do not close it. |
| FCP-PROCESS / NOT_RUN | Execute integrated Graph/CHECK positive and refusal fixtures. | Same tuple/artifacts; Linux Graph success, non-Linux Graph refusal, signed Linux CHECK and relevant non-Linux controls; record selected/executed/pass/skip/leak counts and exact output oracle. |
| FCP-INSTALL / EXTERNAL_PREREQUISITE | Obtain release-owned signed x86_64 profile, root-owned immutable slots, independent approval/key and active generation. | Actual installed CHECK proof. Generic aarch64 artifacts and scripted daemon evidence do not satisfy this gate. |

Execution entries: downstream `just test-gqlang-v3-check-linux-process` and
`just test-gqlang-v3-unsupported-host-process`; Graph selectors and counts are
owned by `tools/testing/suite-registry-v1.yaml` and
`crates/quanta-cli-server/tests/server_socket_daemon_e2e.rs`.
Confirm current selectors and prerequisite profile before running them.
Compile/bootstrap precede process proof; installed trust is a CHECK prerequisite.

## Substrate verification

D1–D7/F1/D6 implementation checkpoints are retired into ADR-0005. The following
are unfinished proof scopes, not instructions to reimplement those changes.

| ID / state | Owner and remaining acceptance |
| --- | --- |
| CHECKED-CALLBACK-FIRST-CAUSE / PROOF_PENDING | Source repair and fixtures integrated `aac43629734`: actual ConsumerStopped disposition through common/infra owners, Kernel success-only metadata postcheck. R54 accepted common callback2 and Kernel finisher5; parser callback chronology remains pending on the successor compile closure. Preserve first recorded cause and success custody checks. These focused scopes do not close all consumers. |
| D1-PROOF / INCOMPLETE | `quanta-adapters-shape`, runtime SDK and converter: actual factory/export plus pure/PTA-derived/disabled/Cost feature combinations and both Shape gates. Latest `07093dbe72a` shares Shape/Cost allocation-reference projection through `quanta-contract-pta-result-shared`; R64 accepted Shape20 and Cost heap20 with independent allocation/truncation result oracles. Runtime/gates and no-Java/Java feature closure remain open. Preserve port/PTA authority and exact output. |
| D2-PTA-PROOF / INCOMPLETE | `codegraph-graph-scc` owns calculation; graph-utils reexports compatibility paths. Fixed-pair R75 accepts core whole-lib20/20 and facade whole-lib1/1 with finished, hash-verified publication on Semantica `f574fa82bf1` (source digest `8c32be4f140bab2bea0c25ce4e1274a7cfe3c835a71b3c42998fb20599ed308f`; locators `/Users/songmin/.codex/task-evidence/oct8-f574-pair/core-scc-f574-r1.json` and `/Users/songmin/.codex/task-evidence/oct8-f574-pair/scc-facade-f574-r1.json`). Resolve actual PTA adapter/Andersen/runtime-analysis proofs of order/IDs, closure, admission and typed stop. Old owner45's graph-utils `scc` selector is zero-case. Tarjan10 source includes PTA adapter/Andersen; execution is unverified. Earlier compile RED stays historical. |
| D2-TAINT-PROOF / INCOMPLETE | `quanta-taint-solver` and contract tests: successor-only domain, order/IDs/resource/stop controls, Tarjan gate and explicitly selected ignored `deep_chain_50k`. A zero-test selector fails. |
| D2-CALLER-COVERAGE / PROOF_PENDING | Account for eleven SCC consumer scopes omitted by historical45/Tarjan10: current-target planner, dominance, concurrency engine and schema, navigation import graph, rule execution, metrics max-depth, runtime community clustering, distinct `codegraph-import-graph` (`daf8bb00bfa`), `codegraph-core-cfg` call-graph schema (`6a795b466f2`, including stats/Zipper consumers) and abstract worklist (`90e6bb0ea3d`). Source reuse is integrated. Reuse accepted matching receipts or resolve actual owner requests with independent order/identity/resource/refusal oracles; freeze final cardinality afterwards. R63 rule34 and R65 dominance3/concurrency38/domain6/metrics14/core-CFG30 retain their earlier frozen source. R66 accepted import graph32 and actual taint-Zipper32 there. R72 accepts current f574 abstract worklist77, including its four SCC witnesses and publication; do not assign a second run. Current-target, navigation and community stay blocked; other final-input custody remains open. Existing D1 Shape and D2 PTA/taint owners are not duplicate assignments. |
| D3-PROOF / INCOMPLETE | `quanta-adapters-dfg` PortDFG and actual taint promotion: remaining canonical owner requests. Pack16 already has a historical canonical result; earlier DFG/taint compile RED executed zero cases. |
| SUBSTRATE-POLICY / INCOMPLETE | Complete affected mandatory gates and consumer feature/dependency checks. Topology5 and scoped GOD retirement are integrated; earlier global Honesty/GOD RED is historical and current results are unverified. Resolve reproduced attributable baseline/identity drift without global reapproval, allow growth or scope reduction. |
| SUBSTRATE-PRODUCT / UNVERIFIED | Aggregate F1/D6 and D7 consumer acceptance. Python/component evidence retains its original sources. Reuse FCP-PROCESS/FCP-INSTALL runs for the same product boundary; do not schedule a second daemon/installed proof. |
| CHECKED-DEFUSE / PROOF_PENDING | Python checked extraction is integrated `27d9505015e`; production retired-carrier getter census is now0 across the inventoried owners. Its145 package files and manifest match the R49 frozen source. Source migration is complete; actual owner walk/control/binding proof remains open. Rust/Java/C/Kotlin/Go/TypeScript cuts are already integrated; no new migration writer. |
| CHECKED-LANG-SUPPLIERS / PARTIAL | All five inventoried plugin migrations are source-integrated; TypeScript joined C/Kotlin/Rust/Go as `b32361e9d83`. R67 Rust76 lacked the required retained regression because it was raw-string fixture text. `f574fa82bf1` activates it. R70/R71 now accepts whole ordinary libraries Rust77/C57/Go54/TypeScript120/Kotlin43 with32 required witnesses across these libraries and SCV2; active dependency closures match that committed main. Remaining work is controlled-carrier refusal and custody reconciliation after the coupled original-output source handoff. Live ParseResult getters retain their separate API; the public trait has no current parameter. |
| CHECKED-RETAINED-CONTROL / OPEN | General facts ingress still carries erased Native control without original output admission. R74 finds the existing dirty runtime→PTA→DFG original issuer caller and keeper calls; do not create another issuer. Complete the producer's new members_v3 handoff at runtime/SDK callers, admitted PTA workspace transactions and DFG original source-world/publication births; both final boundaries currently refuse UnsupportedOperation. Complete canonical adapter→context→extractor→facts retainer output admission in the same coordinated change. Missing required authority refuses. Shared C/Go/TypeScript cap/overflow mechanics and Kotlin exact source/backend binding are already integrated as f2d85a9/df315e6 with their earlier scoped proofs; they do not establish original grant custody. Prove low-budget/cancel/first-refusal/output lifetime, preserve ordinary unsealed compatibility; current/no-tree has test-support coverage but no demonstrated production caller. |
| CHECKED-CURSOR / PROOF_PENDING | Three common/vendor cursor paths are integrated as `dc2611b96e1`; only their original-issued execution acceptance remains open. Common 7 and Java 139 passed on R38; original issuer's five memory/work/cancel cases previously stopped before execution at DefUse compilation. Retain no-backing-growth and first-refusal oracles on the successor source. Java ten paths, CFG six paths and R43 path/CFG/manifest bundle (`26f16598905`) are integrated; do not import them twice. R44 confirmed 14/5/413/13 on its fixed committed checkpoint, not these five controls or current main. |

Fixed-pair R72 owner observation (2026-10-08): Semantica `f574fa82bf1`, QGLang
`e9a268145ca3`, Index `f60609fe2d28`, and QMP `8ea774894335` form a
clean committed sibling tuple. QBC `codegraph-core-abstract-engine` whole-lib
package-all-features passed 77/77, including four SCC worklist oracles, with
zero ignored or filtered. Its request ID is `96dabfdcf7563603c9ed3686549d24e7`.
On the same source digest `8c32be4f140bab2bea0c25ce4e1274a7cfe3c835a71b3c42998fb20599ed308f`,
`quanta-adapters-parser` all-features lib check reaches RED (exit 101, 105
compiler errors, zero tests); request ID `d3e8b5e5b2506be77ba2660c4b36afc4`.
The worklist publication is verified. Parser's `publication-complete.json` is
absent, so its compile failure is a pretest diagnostic, not finished qualification.
The accepted worklist scope does not qualify PTA, DefUse, or product.
Local immutable locators: `/Users/songmin/.codex/task-evidence/oct8-f574-pair/abstract-worklist-f574-r1.json`
and `/Users/songmin/.codex/task-evidence/oct8-f574-pair/parser-f574-owner-r1.json`.

Semantica Rust acceptance uses `scripts/quanta-build-cli` owner resolve/run.
Record execution root, recipe, features, source/dependency pins, actual counts
and immutable receipts. Old write sets/commands and raw JSON receipts can be
recovered from [ADR-0001](../adr/0001-frontend-campaign-consolidation.md).
Core/adapter proof precedes final mandatory gates; product requirements do not
turn every pure helper run into a full installed qualification.

## Downstream consumer execution acceptance

F2/F7 production code is integrated. Remaining obligations include execution
proof. R49 adds real SDK deadline regressions; their execution is pending:

| ID / state | Owning source and required acceptance |
| --- | --- |
| F2 / PROOF_PENDING | Run complete Native minimal/full registry and mutation owners, including admitted context → worker → Preview backend, exact operation parity and provider-off refusal. Preserve PreviewPatch and missing Apply-issuer refusals. |
| F7 / ACCEPTANCE_OPEN | Run existing SDK mapper five (`sdk::abstract_interpret_builder::non_query_error_tests`), plus the existing engine/lowering/adapter/wire behavioral owners. R49 authors real fluent SDK timeout0 and literal normal-output tests for Sign/Interval/Constant; these tests await owner execution. The existing mapper five do not prove default30s/three domains/zero-past/final publication behavior. Preserve the adapter-start cooperative deadline contract. R64 JSON projection13/13 passed on the independent frozen tuple; this does not execute whole DurableReplay. Revalidate original output-custody eight and quarantine after affected memory-owner changes on the frozen tuple. |

F2 and F7 acceptance remain independent; D2 cutover is not a prerequisite for
the deadline repair. Native Slice/product compatibility remains covered by
SUBSTRATE-PRODUCT rather than reopening the completed F4/F9 code work.

Excluded historical candidates remain excluded: F3 has no demonstrated public
token issuance/loss, F8 has no demonstrated startup socket leak, and Graph/CHECK
detail loss or GraphRAG reshaping was not proved to violate a current public
success contract. Optional refactors and stale severities are not open defects.

## Proposed code-query work

State: Proposed, all IDs below remain open. [ADR-0007](../adr/0007-typed-code-query-contract-proposal.md)
owns proposed semantics and independent oracles; [ADR-0006](../adr/0006-code-query-evaluation-boundary.md)
owns evaluation/dependency policy. V3 receipts do not implement or qualify V4.
The first milestone is an installed public Java query joining two real facts.

Dependency spine: `A00 → A01-F → A02 → A03 → (A04 + A05) → A06`.
A06-P is its deterministic physical baseline. Then A07/A08-Q, A11→A12 and
A13-C can follow their own operator gates; A09+A05 precede A10.
A14 compares only implemented/qualified gates; A15 follows that comparison.
Numeric order alone is not a prerequisite. Recursion/flow gates apply only when used.

| ID | 소유자 / 선행 | 산출물 | 완료 증거 |
| --- | --- | --- | --- |
| A00 | 양쪽 / 시작 | Java corpus·독립 label·query/model 정책; CodeQL CLI/extractor/database/query/model pin | calls/cycle/duplicate/absence/alias/sanitizer/unknown-call의 기대값과 측정 범위 고정 |
| A01-F | Semantica fact/source / A00 | 두 실제 fact adapter; schema·nominal key·closure·coverage·lookup/pushdown 계약 | 공통 key로 source-sealed row 발행; 외부 closure·잘못된 schema·불완전 owner 거절 |
| A02 | 양쪽 / A01-F | ADR-0007의 set/bag·tuple identity·dedup 위치·Option/negation/recursion/aggregate/order/path/failure 법칙과 독립 oracle | false·Incomplete·unsupported·resource failure 구분; projection/union 중복과 위험한 변환의 반례 보유 |
| A03 | frontend-contract + downstream result/consumer / A02 | V4 전체 rule/logical 계약과 별도 Result/Completion 계약; ID·capture·type·DAG·SCC validator, 지원 profile | 독립 bytes/vector·paired pin 일치; binding/basis 불일치·구버전·미지원 정책 거절; non-success를 complete answer로 해석 불가 |
| A04 | QGLang / A03 | 모든 head/body term·typed variable·occurrence·binding/correlation·조건·polarity·대안과 span을 보존하는 lowering | source→IR가 oracle과 일치; unbound/negative cycle/value-generating recursion·크기 초과 거절; physical bound/free 도출에 필요한 정보 유지 |
| A05 | Semantica ingress/provider / A01-F+A03 | artifact에 source·key·coverage/model·capability·예산 결합; 필요한 owner만 준비 | public admission 성공과 source/coverage/capability 거절; 작업 전체가 한 요청에 귀속 |
| A06 | Semantica physical/result / A04+A05 | reference evaluator, 기존 연산자 호환성표, public typed join·QueryResultV4 전송 | 설치 SDK/daemon 결과가 tuple oracle과 일치; memory/drop/cancel과 prefix→terminal 검증; coverage 없는 부재·전체 count 거절 |
| A07 | Semantica recursion / A04+A06 | typed finite domain·strata·전체 body 실행; dedup 후 main과 차집합한 new/delta | base/empty/cycle/mutual/nonlinear/negation이 fixed-point oracle과 일치; 동일 tuple의 복수 derivation은 재삽입 안 됨; 수렴 전 cap은 non-success |
| A08-Q | QGLang+Semantica / A06; 재귀 사용 시 A07 | 필요한 aggregate/sort/limit/cursor·multi-provider query | reorder/duplicate/residual/offset에서 결과·multiplicity·최종 상태 일치 |
| A09 | modules+model/taint owner / A01-F+A03 | Java pack + FlowProfileV4/FlowCoverageV4; alias/call-resolution/summary·unknown-call 정책·model pin·가정·absence authority | labeled sanitizer/alias/state/ambiguous dispatch 일치; 누락 model/summary를 식별; 보수적 fallback과 coverage gap 구분 |
| A10 | flow/check+solver/SDK / A05+A09; 필요한 result 연산자 | 새 flow/check 계약을 기존 solver·결과 경로에 연결 | 설치 positive/sanitizer-negative와 실제 context/state/source-valid witness; model gap·cap·worker 오류가 clean 판정으로 바뀌지 않음; profile/scope 밖의 안전성 주장 거절 |
| A11 | 양쪽 / 최소 A06부터 | logical/physical ID·선택 이유·추정/실제 rows·work/memory·delta·model·optimizer stop | 원인을 owner까지 추적; redaction·evidence 생성 크기 제한. rewrite 설명과 flow witness 구분 |
| A12 | Semantica / 해당 실행 연산자+A11 | 합법적 접근/join 대안·guarded rewrite·demand/SIPS의 별도 검증·동일 workload 비교 | 결과/order/witness 일치; bad stats/skew/fanout/재귀 증폭에서 bounded work·memory·typed 실패; 이후 평균 비용 개선 |
| A13 | 양쪽 / A06부터; 유지할 연산자의 gate | SCC/strata invalidation+recompute 먼저, 이후 +Δ/-Δ·replay-safe epoch·lease·증분 유지 | 매 공개 epoch가 fresh run과 일치; replay/delete/anti-join/model/retention 실패가 stale 성공을 만들지 않음 |
| A14 | 양쪽 / 비교에서 주장하는 gate | 동일 label/source/model의 CodeQL 비교, 미지원·품질·build/query/RSS 공개 | 사전 정의한 장점·허용 회귀 기준 통과. Index·증분은 구현·검증한 경우만 비교 |
| A15 | 양쪽 / A14 | 언어·framework별 fact/model/query 확대 | 추가 family마다 독립 label·실제 provider·설치 proof·새 비교 |

### Optional Index acceptance

- **A01-I / Semantica+Index:** publish analysis-source closure ↔ exact corpus relationship, workspace overlay/incarnation and snapshot serviceability.
- **A08-I / A01-I+A05+used operators:** candidate→canonical fact ID adapter retaining window/continuation/snapshot.
- Gate: admitted exact-plan Index on/off results match. Reject corpus mismatch/expiry/bad candidates; active-head movement cannot retarget held reads. Approximate/top-k/capped windows cannot become exhaustive input.

### Optimizer and incremental sub-batches

| 배치 | 작업과 gate |
| --- | --- |
| A12-P | Provider가 지원하는 scan/lookup/join 후보만 생성. 필수 schema/source/물리 속성을 검사하고 unknown/nonfinite/overflow 비용·canonical tie-break 정의 |
| A12-R | Rewrite별 독립 결과 oracle과 금지 반례. residual/limit·semi/anti·correlation·Option·multiplicity 보존. 미지원 outer/volatile 의미는 거절 |
| A12-D | A07 이후 positive recursion의 bound/free를 도출. Demand specialization과 SIPS/early filtering을 별도 변환으로 검증. Entry가 제한된 그래프에서 full-closure oracle과 scoped answer 일치·charged work 비교; negation/aggregate/flow 확장은 별도 proof 전 거절 |
| A12-M | planning+execution latency, estimation error, work·retained memory 비교. Bad stats/skew/fanout·반복 증폭에서 reservation/cancel/bounded work 검증. 선택적 optimizer cutoff는 유효 계획과 stop reason을 반환할 수 있음; request cancel/resource failure는 terminal |
| A13-C | source/provider/schema/query/contract/model dependency와 SCC/downstream strata에 따라 invalidate+fresh recompute. Query DX는 별도로 진행 |
| A13-I | owner별 기여·+Δ/-Δ·tuple equality/multiplicity·checked weight·replay-safe sequence/atomic replace·epoch input/output seal 정의; signed weight를 public negative row로 노출하지 않음 |
| A13-L | old query/cursor/evidence lease·expiry·shared memory owner·compaction 제한. 보존 압력은 명시적 거절/실패; held history를 몰래 제거하지 않음 |
| A13-O | duplicate/reorder/restart·owner-local retraction·두 재귀 경로의 순차 삭제·SCC 재계산·anti-join 첫 RHS 추가/마지막 RHS 삭제·partial coverage·model change·lease expiry를 매 epoch fresh run과 비교 |

A13-I/L/O form one publication gate. No DataFusion/Soufflé/Differential/memo/e-graph
adoption is selected without comparable workload, semantics, resource and integration-cost evidence.

Stop conditions: missing source-sealed facts precede grammar growth; absent absence
authority rejects `not`; unsupported solver policy rejects the flow/path combination.
Oracle mismatch stops optimization. Optimizer effort cutoff is distinct from request failure.
Each batch records owner, HEAD/dirty paths, source/contract/model/Index tuple, command,
actual counts and immutable receipt. Small owner rails precede repository `just ci`;
clean committed qualification remains separate.

## Closeout rule

Move an item out of this index when its own acceptance passes on identified
source. Preserve durable decisions in the owning ADR/contract and exact receipts
in source-bound evidence. For this repository, `just ci` is local composition;
`just qualify-local` requires clean committed HEAD. No stale receipt, source
presence, compiler success or Python fixture alone grants product qualification.
