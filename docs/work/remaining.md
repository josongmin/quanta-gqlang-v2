# Remaining work

Updated: 2026-10-07. Latest read-only implementation reconciliation: committed
Semantica `5f76eca7812cdbed3cdd66dd6d0f6533ce9b1c9c`; foreign dirty overlays are
excluded. Source integration and runtime qualification remain separate.
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
Completed repair decisions belong to
[ADR-0005](../adr/0005-substrate-reuse-and-consumer-projection.md#subsequent-integrated-repair-decisions).

## Frontend diagnostic contract

### QG-DIAG

- State: CONTRACT_DECISION; historical IR-08/F-2, not a completed repair.
- Owner: diagnostics stdio and its consumer. Current
  [wire DTO](../../crates/quanta-gqlang-v2-diagnostics-stdio/src/server.rs) contains
  primary locations; the [handler](../../crates/quanta-gqlang-v2-diagnostics-stdio/src/handlers/v3_diagnostics.rs)
  omits secondary locations and wraps authority codes into `authority_error`.
- Acceptance: decide the private protocol's required fields from an actual
  consumer. Either document that narrower contract, or version the wire and
  preserve exact secondary locations and typed authority identity with process
  positive/rejection oracles. Do not mark the existing wire as repaired.

## V3 downstream product proof

Owner: Semantica SDK/runtime/release. Producer maintenance is integrated.
Existing fixtures and lane registration are integrated; execute their oracles
before claiming completion. Broader AG5/IRC releases remain downstream-owned.

| ID / state | Required work | Acceptance |
| --- | --- | --- |
| FCP-COMPILE / UNVERIFIED | Re-freeze original-session/QMP pin and SDK/PTA/runtime-analysis integration on committed main. Earlier attempts failed before actual process execution. | Canonical product build and SDK owner rails on one source tuple; report actual current diagnostics, not the old 31/5 error counts. |
| FCP-PTA / DIAGNOSIS | Verify the seed-consumption lifetime repair against the first `PtaOccurrenceAuthorityRefused` boundary. | Current owner/bootstrap succeeds, or a reached refusal identifies the exact remaining owner repair. Source changes alone do not close it. |
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
| CHECKED-CALLBACK-FIRST-CAUSE / PROOF_PENDING | Source repair and fixtures integrated `aac43629734`: actual ConsumerStopped disposition through common/infra owners, Kernel success-only metadata postcheck. Execute owning callback chronology oracles when authorized; preserve first recorded cause and success custody checks. Authored fixtures/source integration are not execution proof. No duplicate bridge repair remains assigned. |
| D1-PROOF / INCOMPLETE | `quanta-adapters-shape`, runtime SDK and converter: actual factory/export plus pure/PTA-derived/disabled/Cost feature combinations and both Shape gates. Preserve port/PTA authority and exact output. |
| D2-PTA-PROOF / INCOMPLETE | `codegraph-graph-scc` now owns calculation; graph-utils reexports compatibility paths. Resolve core/facade and actual runtime-analysis/taint plus moved PTA adapter/Andersen caller proofs of order/IDs, closure, admission and typed stop. Old owner45's graph-utils `scc` selector is now zero-case; successor map is47 if other rows remain. Current Tarjan10 source includes the PTA adapter and Andersen; execute it and repair the historical map. Review added SCC consumer coverage, including concurrency deadlocks, before final cardinality. Earlier compile RED stays historical. |
| D2-TAINT-PROOF / INCOMPLETE | `quanta-taint-solver` and contract tests: successor-only domain, order/IDs/resource/stop controls, Tarjan gate and explicitly selected ignored `deep_chain_50k`. A zero-test selector fails. |
| D3-PROOF / INCOMPLETE | `quanta-adapters-dfg` PortDFG and actual taint promotion: remaining canonical owner requests. Pack16 already has a historical canonical result; earlier DFG/taint compile RED executed zero cases. |
| D4/D5-CUSTODY / UNVERIFIED | SCV2 leaf34 and isolated solver225+API-absence2 have canonical historical results. Rebind unchanged owners or rerun changed inputs for the final integration tuple; do not repeat the old manual-only gap as if canonical results were absent. |
| SUBSTRATE-POLICY / INCOMPLETE | Complete affected mandatory gates and consumer feature/dependency checks. Topology5 and scoped GOD retirement are integrated; global Honesty/GOD ratchets remained RED. Resolve attributable baseline/identity drift without global reapproval, allow growth or scope reduction. |
| SUBSTRATE-PRODUCT / UNVERIFIED | F1/D6 and D7 have Python/component evidence; native daemon and product compatibility/installed acceptance remain distinct. Reuse the FCP frozen tuple when the same product boundary is tested. |
| CHECKED-DEFUSE / OPEN | Committed Python/TypeScript frontdoors retain two retired carrier getters. Python has foreign WIP under its existing owner; TypeScript is a separate source cut after refreshed claims. Rust (`ed94af8`) and Java (`e14eeda`) joined integrated C/Kotlin/Go; do not redevelop them. Full walk/control/binding and execution acceptance remain distinct. |
| CHECKED-LANG-SUPPLIERS / OPEN | Only TypeScript's two from-tree entrypoints retain retired carrier calls. Kotlin (`ecc0c57`) and Rust/Go (`ed94af8`) joined integrated C. TypeScript must migrate its actual helper calculations and preserve quarantine/output/order/authority; other language owners require proof. The public trait has no current parameter: ordinary read and controlled-carrier refusal remain separate. |
| CHECKED-RETAINED-CONTROL / OPEN | Implement the missing adapter→context→extractor original-output bridge using existing admission interfaces; centralize Native source/backend policy. C/Go scalar snapshots have small duplicate 128 MiB cap/overflow ledgers and fact calculation after the checked callback without current; resolve duplicate mechanics within the actual birth/work/custody cut, keeping distinct grammar projections. Kotlin needs exact Native retained admission. Add sealed mismatch/missing authority and original low-budget/cancel/first-refusal/output-lifetime oracles. Preserve ordinary unsealed compatibility; current/no-tree uses existing test-support context, with no claimed production path. |
| CHECKED-CURSOR / PROOF_PENDING | Three common/vendor cursor paths are integrated as `dc2611b96e1`; only their original-issued execution acceptance remains open. Common 7 and Java 139 passed on R38; original issuer's five memory/work/cancel cases previously stopped before execution at DefUse compilation. Retain no-backing-growth and first-refusal oracles on the successor source. Java ten paths, CFG six paths and R43 path/CFG/manifest bundle (`26f16598905`) are integrated; do not import them twice. R44 confirmed 14/5/413/13 on its fixed committed checkpoint, not these five controls or current main. |

Semantica Rust acceptance uses `scripts/quanta-build-cli` owner resolve/run.
Record execution root, recipe, features, source/dependency pins, actual counts
and immutable receipts. Old write sets/commands and raw JSON receipts can be
recovered from [ADR-0001](../adr/0001-frontend-campaign-consolidation.md).
Core/adapter proof precedes final mandatory gates; product requirements do not
turn every pure helper run into a full installed qualification.

## Downstream consumer execution acceptance

F2/F7 production code is integrated. Remaining obligations include execution
proof and missing real SDK deadline regression coverage:

| ID / state | Owning source and required acceptance |
| --- | --- |
| F2 / PROOF_PENDING | Run complete Native minimal/full registry and mutation owners, including admitted context → worker → Preview backend, exact operation parity and provider-off refusal. Preserve PreviewPatch and missing Apply-issuer refusals. |
| F7 / ACCEPTANCE_OPEN | Run existing SDK mapper five (`sdk::abstract_interpret_builder::non_query_error_tests`), plus the existing engine/lowering/adapter/wire behavioral owners. Add real fluent SDK deadline result coverage; the current mapper five do not prove default30s/three domains/zero-past/final publication behavior. Preserve the adapter-start cooperative deadline contract. Revalidate original output-custody eight, JSON and quarantine after affected memory-owner changes on the frozen tuple. |

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
