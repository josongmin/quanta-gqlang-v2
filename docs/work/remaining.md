# Remaining work

Updated: 2026-10-10. QGLang source readback:
`25fad76cf5b4510244157c2e009b4dc37aa958ff` (clean before this documentation pass).
Current behavior belongs to source/contracts; decisions belong to ADRs. This
index contains unfinished acceptance and a separate Proposed roadmap.

## Frontend and paired acceptance

D01–D07 source work and its caller invariants are consolidated in
[ADR-0009](../adr/0009-frontend-owner-dedup-and-execution-contract.md).
QFC-01/03/06 are retired source tasks. QG-DIAG is the resolved
[ADR-0008](../adr/0008-private-diagnostics-primary-location-contract.md) contract
and is not an open repair.

- **QFC-02 / OPEN_DOWNSTREAM / Semantica E13:** reconcile bootstrap full revision
  `9060e46997dd9c62080a18bdfae06f8c162442e2`, current producer closure (including
  execution-contract), subtree/vector policy and compatible consumer compile/
  contract-vector proof. Updating expected pins alone is insufficient.
- **QFC-04/05 / RECEIPT_DERIVED:** standalone completion follows the exact current
  committed HEAD's schema-3 receipt under `target/qualification/<HEAD>/` and
  [qualification procedure](../../tools/qualification/README.md). Incoming
  `25fad76…` has verified local qualification (1,529 Nextest, zero skips/leaks);
  this documentation overlay does not inherit that clean-HEAD result.

No standalone frontend code defect was established by this reconciliation.
Paired/product proof and Proposed V4 are separate scopes.

## Downstream owner and source boundary

[Semantica Oct-10 compiled](../../../semantica-codegraph-v2/docs/handoff/oct-10/oct-10-compiled.md)
is the single downstream execution map: E00–E18, Parent18 crosswalk (§8.3),
original25 and cross-cut/product gates (§9), eleven SCC consumers (§10.2).
Those overlapping sets are not additive implementation counts. Existing owners
retain their write/handback boundaries; old A/B/C schedules grant no new writers.

Read-only committed Semantica `7836ed9c03e98126f2dece066c7814de3027bfaf`
contains all ten recorded R356 postimages byte-for-byte, also matching live paths.
The old ten-path “uncommitted/not integrated” claim is retired. This is exact
source integration only; no new Rust/QBC/runtime/product execution occurred here.
[ADR-0005](../adr/0005-substrate-reuse-and-consumer-projection.md#r356-source-integration-readback--2026-10-10)
retains that observation. Other current implementation gaps must be checked at
the compiled-map owner before assignment; historical WIP/error counts are not
current defect counts.

## Substrate verification

The original 18 IDs and recorded states are carried forward below. Their closure
still needs current compatible owner evidence; this table does not requalify old
passes or classify every downstream task as missing implementation.
D4/D5-CUSTODY is scoped accepted in ADR-0005 and is not reopened. Execution
oracles and source-freeze rules are in the [proof handoff](substrate-repair-checkpoints-2026-10-07.md).

| ID | Priority / recorded state | Existing epic / remaining acceptance |
| --- | --- | --- |
| CHECKED-LANG-SUPPLIERS | P1 / PARTIAL | E02/E07/E13: Controlled language positive/refusal, source/output custody and actual feature owners; ordinary accepted migrations are not new implementation. |
| CHECKED-DEFUSE | P0 / PARTIAL | E07/E13: Genuine nonempty facts→canonical final SSA/DFG→adapter output, exact identity/order, no partial publication, retained keeper and final refund. |
| CHECKED-RETAINED-CONTROL | P0 / OPEN | E02–E12: Genuine member/provider/cohort/inventory; same-operation output, selected PTA and full public fixture; canonical binding partial custody; SDK generation/receipt and persistent packet/memo publication. |
| F7 | P1 / ACCEPTANCE_OPEN | E11/E13: Actual SDK timeout0 and literal normal Sign/Interval/Constant results, default/iteration distinction, cooperative final publication and typed errors; mapper five alone is insufficient. |
| CHECKED-CALLBACK-FIRST-CAUSE | P1 / PROOF_PENDING | E01/E02/E13: Actual Parser/common/Kernel chronology: outer Source first cause, complete consumer/losing errors retained through finisher, no later polling/work/bytes, final refund. |
| CHECKED-CURSOR | P1 / PROOF_PENDING | E01/E13: Explicitly execute all five Original-issued traversal memory/work/cancel controls; preserve no-backing-growth and first-refusal assertions. |
| D1-PROOF | P1 / INCOMPLETE | E13: Actual Shape factory/export and SDK/Cost/PTA-derived/provider-off feature matrix with exact allocation/result projection. |
| D2-PTA-PROOF | P1 / INCOMPLETE | E05–E10/E13: Current PTA/Andersen/precise Runtime SCC consumers: partition/order/IDs, domain closure, admitted work/storage and typed stop. |
| D2-TAINT-PROOF | P1 / INCOMPLETE | E13: Taint successor-only domain/public contract and explicitly selected ignored `tarjan_iterative_deep_chain_50k`; nonzero execution. |
| D2-CALLER-COVERAGE | P1 / PROOF_PENDING | E13: All eleven affected SCC consumers with independent partition/order/witness/resource oracles; compiled plan section 10.2 owns the current path map. |
| D3-PROOF | P1 / INCOMPLETE | E07/E13: Retained DFG→PortDFG/MemSSA→real taint promotion on the compatible tuple; historical pack proof is separate. |
| SUBSTRATE-POLICY | P1 / INCOMPLETE | E13: Affected mandatory Honesty/God/AST/boundary/feature/manifests/locks; no allowance growth, baseline reset or scope reduction to pass. |
| F2 | P1 / PROOF_PENDING | E11/E13: Whole Native minimum/Full registry and mutation owners, exact ID/service parity, admitted context→worker→Preview, provider-off and absent Apply-issuer refusals. |
| FCP-COMPILE | P0 / ACCEPTANCE_PENDING | E01/E13: Freeze complete compatible supplier/caller feature closure; non-Java/Java/SDK/Runtime owner compilation/tests, finished QBC publication and coupled committed-source continuity. |
| FCP-PTA | P1 / REVERIFY | E05–E10/E13: Reproduce actual current bootstrap; repair only the reached owner cause. Historical refusal/diagnostic counts are not current defects. |
| FCP-PROCESS | P1 / NOT_RUN | E13: Same-source/artifact Graph/CHECK processes: exact positive/refusal outputs, channel/exit, timeout kill/reap and no required skips/leaks. |
| FCP-INSTALL | P1 / EXTERNAL_PREREQUISITE | E13: Release-owned signed Linux CHECK profile, root-owned immutable slots, independent approval/key, active generation and actual installed positive; non-Linux refusal is separate. |
| SUBSTRATE-PRODUCT | P1 / UNVERIFIED | E13: F1/D6/D7 rollup reuses the same qualified process/artifact/source tuple; component fixtures/readiness do not close it. |

Public selected-ingress dependency: genuine CFG/facts → SAME Braun SSA →
authenticated final DFG/key → selected PTA adapter + actual Runtime grant →
public fixture/consumer → compatible owner proof/integration → process/product.
Canonical binding partial custody and direct final-generation oracles can
proceed at their own prerequisites; no second SSA/DFG pipeline is assigned.

R356 P1–P6 are existing E06/E07/E10/E13 subscopes: final DFG/key, same SSA export,
canonical binding partials, selected PTA/Runtime grant, public fixture and shared
parent function/God debt. Retain genuine recorded destination D, not an arbitrary
admission parameter. Recheck source before assigning implementation; the
unaccepted `original_final_dfg_backing_v3` wrapper is not a new default design.

## Downstream consumer execution acceptance

F2/F7 remain independent. Native Full uses the actual distribution rather than
provider-only Interactive registration. Preview backend reachability does not
supply Apply's missing terminal/replay issuer. F7 mapper conversion does not
supply actual fluent SDK deadline behavior. Required owner witnesses may not
be ignored or replaced with compiler-only success. Old F3/F8 candidates remain
excluded absent a reachable reproducer.

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

Retire an item only when its own acceptance passes on identified source. Keep
one downstream execution map, one unfinished index and durable ADR/contract
owners. Historical plans/selectors/receipts are recoverable at
[ADR-0001's Oct-10 baseline](../adr/0001-frontend-campaign-consolidation.md#2026-10-10-reconciliation).
Repository `just ci` proves its recorded overlay; `just qualify-local` requires
clean committed HEAD. Source presence, focused passes and old receipts do not
establish paired/runtime/product qualification.
