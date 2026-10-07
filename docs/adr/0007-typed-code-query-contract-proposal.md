# ADR-0007: Typed code-query contract proposal

- Status: Proposed
- Origin: 2026-10-04 RFC; consolidated 2026-10-07
- Scope: language, logical V4 and downstream Result/Completion proposal
- Acceptance: [A00–A15 and optional batches](../work/remaining.md#proposed-code-query-work)

## Current source and ownership

QGLang HEAD `50d4a6710b542eaf51a2e2ee70c4db5acb5a09d6` exposes the
[V3 compile facade](../../crates/quanta-gqlang-v2-frontend-contract/src/lib.rs).
No `QueryResultV4`, `CompletionBasisV4` or `FlowProfileV4` type is implemented
in its crates; [closed sema](../../crates/quanta-gqlang-v2-sema/src/document/query_materialization.rs)
explicitly rejects authored `result_shape`. Existing V3 CHECK dispatch and
separate downstream planners/recursion/solvers do not implement this proposal.

목표는 실제 code fact의 typed join, finite recursion, model 기반 flow와
검증된 finding/witness다. 첫 milestone은 설치된 공개 Java 질의의 두 fact
join이다. Index·optimizer·재귀·flow는 첫 join의 선행 조건이 아니다.

QGLang은 syntax/HIR/sema/logical 계약, Semantica는 source/provider admission,
physical planning·실행·solver·결과를 소유한다. Index는 선택적 candidate 접근이다.
Logical bytes에 physical algorithm·provider handle·runtime/Index pin을 넣지 않는다.
기존 owner를 연결하며 새 proxy 레포/프로세스나 전체 engine 교체는 선택하지 않는다.

## Proposed language and compatibility

- `relation`: finite typed tuple 집합, positive recursion과 stratified negation.
- `query`: join/filter/project/group/order/limit, 기본 set. `rows query`는 명시적 bag이며 recursive body에서 제외한다.
- `flow`: context/alias/summary·유한 state·model 정책. Graph BFS와 별도 의미다.
- `check`: read-only finding·location·rule/model identity·witness. Mutation은 기존 Command 경로다.
- `find`: 지원 graph/flow 계약으로 lowering하는 surface.

A02/A03은 set/bag·completion/absence·flow profile/coverage 법칙을 먼저 고정한다.
A03/A04는 typed head/body·occurrence·variable·binding/correlation·SCC 정보를
보존한다. Physical bound/free는 여기서 도출하며 bitmap이나 signed runtime
update를 frontend 실행 정책으로 고정하지 않는다.

V4 cutover는 DTO·digest domain·producer vector·consumer validator/golden/pin을
함께 변경한다. Old/incompatible bytes를 거절하고 V3 bag 문법은 명시적
변환 또는 migration 오류로 처리한다. 새 record syntax가 기존 result-shape
지원을 뜻하지 않는다. 공개 계약 승격은 paired source/product proof 이후다.

## Proposed invariants

### Fact·scope·authority

Provider는 schema/version, nominal entity/key, source closure, lookup 방향, 정책별 coverage·soundness와 실패 상태를 발행한다. 서로 다른 provider의 문자열 ID가 같다고 join하지 않는다. 통계는 coverage·정확성의 증명이 아니다.

Predicate pushdown은 `unsupported / exact / superset + residual`로 선언한다. Superset은 true match를 누락하지 않고 residual에 필요한 column을 유지한다. 미지원 lookup은 후보로 만들지 않는다.

**결과 endpoint 범위, 중간 traversal 범위, 데이터 접근 권한을 구분한다.** 엔진은 선언된 의미를 계산하기 위해 추가 fact를 읽고 join할 수 있다. 읽기는 허용된 source·권한·예산 안에 있어야 하며, endpoint 필터를 임의로 induced subgraph로 바꾸지 않는다.

Index 사용 시 분석 source↔정확한 검색 corpus의 발행 관계를 검증하고 candidate를 canonical fact ID로 확인한다. 기존 snapshot이 유효하면 active head 이동은 허용한다. 불일치·만료 때 새 active로 몰래 전환하지 않는다. Approximate/top-k/capped window는 exhaustive count·absence·finding 입력의 유일한 근거가 될 수 없다.

### Logical program·의미

Consumer는 operator/relation/body-occurrence ID, 참조·capture scope, schema/arity/type, 일반 DAG와 명시적 SCC/strata를 독립 검증한다. 지원 operator/type/expression/result/flow 정책을 versioned profile로 선언하고 실제 admission/dispatch와 맞춘다. 필수 의미와 advisory hint를 구분한다.

- Rule은 전체 head/body term·조건·polarity·대안 clause를 보존한다. dependency 목록만으로 실행하지 않는다. OR 정규화의 크기·깊이를 제한한다.
- 첫 재귀 subset은 유한 입력 domain·상수에 한정한다. value-generating recursive constructor와 recursive aggregate는 거절한다. 관계별 `main/delta/new`와 모든 재귀 atom occurrence를 다루고 nonlinear/mutual recursion을 검증한다.
- Set relation의 tuple identity는 schema에 따른 전체 logical tuple이다. Projection·대안 clause union·relation 경계에서 중복을 제거한다. Bag query는 중복을 보존하고 명시적 `distinct`만 제거한다. Physical bag 저장은 logical set boundary를 바꾸지 않는다.
- 재귀의 `new_r = distinct(project_head(이번 round의 derivations_r)) ∖ main_r`; 같은 round의 후보도 중복 제거하고 다음 `delta_r = new_r`, `main_r := main_r ∪ new_r`로 진행한다. Equivalent schedule은 허용하되 nonlinear atom 대안의 결과·수렴은 같아야 한다. Provenance 중복을 tuple multiplicity로 세지 않는다.
- `not`·anti-join은 해당 scope/source/정책에서 부재를 정당화할 coverage가 필요하다. 불완전 fact를 빈 관계로 바꾸지 않는다.
- Null은 `Option<T>`다. SQL null·문자열·set/bag로 암묵 변환하지 않는다. global empty `count`는 0이며 다른 empty aggregate는 타입별로 정의한다.
- `limit`은 total order 뒤의 logical 결과에 적용한다. fetch/local-partition/global limit을 구분하고 residual·join·multiplicity·offset을 보존하는 경우만 조기 실행한다.

### Graph·flow·finding

Graph provider는 exact forward/backward adjacency, stable node/edge ID, source와 paged cursor 완료 상태를 제공한다. Endpoint `(source,sink)` 집합, edge/orientation 순서열의 path identity, query derivation multiplicity를 구분한다.

Hop/zero-length/direction/repetition, 대표 1개/전체, 단일/전체 최단 정책은 profile에 명시한다. 첫 profile은 증명된 bounded 대표 정책만 지원해도 된다. 선택 기준과 tie-order를 먼저 고정하고 미지원 조합은 거절한다. Authored hop bound 안의 완전한 탐색과 host cap에 의한 탐색 중단을 구분한다.

Flow는 solver의 context/alias/summary·유한 state 정책과 pinned model에 연결한다. Model은 callable/signature/subtype, arg/return access path, barrier/guard 방향, 우선순위·충돌·unknown-call fallback을 정의한다. Unknown call은 선언된 보수적 fallback을 따르며 자동으로 `Incomplete`가 되는 것은 아니다.

`FlowProfileV4`는 alias/call-resolution/unknown-call/summary 정책, model digest와 absence 정책을 고정한다. Runtime `FlowCoverageV4`는 source·scope·profile에 묶인 model/summary coverage, 누락 unit·cap과 가정을 기록한다. Soundness는 선언된 분석 정책·가정에 대한 주장이지 Boolean “안전함” 필드가 아니다. 완료된 계산에서 “profile P로 flow를 찾지 못함”과 “P의 부재 권한 아래 해당 scope에 flow 없음”을 구분한다. 필요한 model/summary coverage가 부족하면 부재 판정을 발행하지 않는다.

Required witness는 같은 source/model에서 call/return·state·edge/summary와 숨겨진 설명 단계를 검증한다. 일반 graph path를 flow witness로 대체하지 않는다. Finding key와 witness 선택을 분리하고 cross-generation key 안정성을 별도 증명한다. 완전한 finding scan만 성공한 SARIF로 내보낸다.

### Public Result/Completion 타입

아래는 A03에서 고정할 **제안 schema**다. Binding은 downstream admission이 발행한다. Frontend logical bytes에 runtime source/model handle을 추가하는 변경이 아니다.

```rust
struct ExecutionBindingV4 {
    query_id: QueryIdV4,
    source_id: IssuedSourceIdV4,
    model_digest: Option<ModelSetDigestV4>,
    profile_id: ProfileDigestV4,
}
enum QueryResultV4 {
    Complete { binding: ExecutionBindingV4,
        basis: CompletionBasisV4, answer: CompleteAnswerV4 },
    Incomplete { request_id: RequestIdV4,
        observed: Option<BoundObservationV4>,
        reasons: NonEmpty<IncompleteReasonV4> },
    Unsupported { request_id: RequestIdV4, requirement: UnsupportedRequirementV4 },
    Failed { request_id: RequestIdV4,
        binding: Option<ExecutionBindingV4>, failure: ExecutionFailureV4 },
}
```

`CompleteAnswerV4`는 typed rows/findings 또는 검증된 output reference다. `CompletionBasisV4`는 질의 scope·coverage·가정과 필요한 경우 owner가 발행한 absence authority를 binding에 연결한다. `BoundObservationV4`는 binding과 관찰 prefix를 함께 가진다. Non-success는 observation만 가질 수 있으며 complete answer를 갖지 못한다. Admission 전 실패는 issued source를 꾸며내지 않는다. Reasons/failure는 versioned typed code와 제한된 상세다.

Page event는 관찰값, terminal event는 `QueryResultV4`다. 질의가 요구한 enumeration이나 결과 전송이 미완료면 terminal `Complete`를 발행하지 않는다. Authored limit을 충족한 질의는 나머지 원본 row를 읽지 않아도 완료할 수 있다. Complete 생성자는 owner의 scope/profile/coverage와 required witness 확인을 요구한다. Decoder·SDK도 같은 조건을 검증한다. 부재·finding-clean은 **Complete + 해당 범위의 absence authority**, 정확한 전체 count는 **Complete + 해당 입력 범위의 exhaustive coverage**를 요구한다. 제한된 ranked query의 완료나 반환 prefix count를 전체 부재·전체 count로 승격하지 않는다.

### 실행·최적화

Retained join/sort/recursion/evidence는 할당 전 memory reservation과 종료·오류·drop 시 해제를 요구한다. 배치·backpressure·owned task·취소 checkpoint를 제한한다. Spill이 없으면 bound 초과 전에 실패한다.

중간 page는 같은 query/source/model에 묶인 provisional prefix다. 최종 status만 완료를 증명한다. Provider 누락, solver cap/worker 오류, 예산·취소 실패를 완전한 0행·부재 판정으로 바꾸지 않는다. 재귀 tuple cap과 결과 cardinality가 같다는 사실은 수렴 증명이 아니다.

먼저 deterministic admitted baseline을 만든다. 필요한 물리 속성만 요구하고 미상·비유한·overflow 비용과 동률 처리를 정의한다. 이후 작은 합법적 scan/lookup/join 후보부터 비교한다. 선택적 **optimizer 노력 제한**은 유효한 baseline/best-known plan으로 끝낼 수 있으나, 요청 취소·필수 자원 실패를 숨기면 안 된다. 충분한 예산에서는 결과·순서·witness 정책이 같아야 하며 제한된 예산에서는 합법적 계획별 실패 시점이 달라질 수 있다.

Optimizer 목적 순서는 **semantic legality → pathological intermediate/work 방어 → deterministic fallback → 평균 비용 개선**이다. Hard request bound는 나쁜 계획을 싸게 만든다는 뜻이 아니라 폭주를 typed failure로 멈추는 장치다. 초기 개선은 지원되는 hash join/indexed lookup/join order·early filtering부터 검증한다.

Rule별 relational plan과 SCC recursive control을 분리한다. Planner는 보존된 variable/binding/occurrence로 bound/free를 계산한다. Query binding을 재귀 안으로 전달하는 demand specialization이나 SIPS/semijoin early filtering은 서로 구분하고, 의미 보존을 증명한 positive subset에만 적용한다. Negation·aggregate·flow는 별도 증명 전 specialization하지 않는다. 전체 closure와 동일한 scoped answer를 내는지 검사한다. [FlowLog](https://www.vldb.org/pvldb/vol19/p361-zhao.pdf)는 rule IR/control 분리와 robustness·SIPS 참고이며 우리 성능의 증거는 아니다.

### 증분 실행

먼저 dependency invalidation+fresh recomputation을 맞춘다. 유지 실행은 이후 별도 gate다. Owner별 insert/delete 기여와 replay-safe epoch/sequence, 전체 input seal·output 완료, source/model coverage를 함께 확인한다. 재전송이 multiplicity를 중복 반영하면 안 된다.

Runtime operator adapter는 후속 `+Δ/-Δ`를 수용할 수 있도록 tuple equality·multiplicity·dependency를 명시한다. 지금 모든 연산자에 delta API를 구현하지 않는다. Signed weight는 내부 갱신이며 public set의 negative row가 아니다. 삭제는 해당 SCC와 downstream dependent strata를 invalidate/recompute하는 baseline부터 시작한다. Anti-join은 RHS 추가 시 기존 output 철회, 마지막 RHS 제거 시 output 복구를 검증한 뒤 유지 실행한다. [CodeQL diff-informed hook](https://codeql.github.com/codeql-standard-libraries/java/codeql/dataflow/DataFlow.qll/predicate.DataFlow%24Configs%24ConfigSig%24observeDiffInformedIncrementalMode.html)은 변경 관련 source/sink 필터의 예이며 full incremental engine의 증거가 아니다.

Query/cursor/evidence lease와 shared memory owner를 정하고 held history를 compaction으로 파괴하지 않는다. 보존 압력은 새 작업 거절 또는 명시적 만료·실패로 처리한다. Replay/restart·삭제·negation·model change의 매 공개 epoch가 fresh run과 같아야 한다.

## Independent acceptance oracles

- Duplicate key·projection/union set dedup와 explicit bag multiplicity, correlated EXISTS/NOT EXISTS의 false conjunct, cross-provider key와 partial fact.
- Residual이 앞 후보를 제거하는 limit, offset·reordering, reserve 실패·취소/drop 후 해제. Page 이후 오류를 exact count·absence·0행으로 바꾸지 않는다.
- Forged binding/basis, scope/profile mismatch, incomplete cursor와 authored-limit completion을 decoder와 SDK에서 독립 구별한다.
- Graph: endpoint filter와 traversal domain, parallel edge/self-loop/cycle, repeated node/edge, ties·zero/exact hop·paged/inverse adjacency·derivation duplicate. Endpoint true·required witness 실패·host cap을 구별한다.
- A06-P는 기존 planner/lowering/executor의 bag/set·Option·match/post-join filter·correlation·source·budget을 독립 결과와 비교한다. 필요한 order/partition 속성만 요구한다.
- 매 공개 incremental epoch를 fresh run과 비교한다. Maintained publication은 A13-I/L/O가 함께 통과한 뒤 허용한다.

## Evidence and deferred decisions

[ADR-0006](0006-code-query-evaluation-boundary.md)이 reference pins·비교 순서·
dependency 선택 기준을 소유한다. CodeQL 비교는 동일 독립 label/source/model,
cold/warm build/query latency·RSS·미지원 범위를 기록한다. Source 유사성이나
기존 V3 receipt는 새 V4 기능·성능·제품 qualification이 아니다.

첫 milestone에서 full SMT/symbolic execution/storage 교체, lattice language,
arbitrary recursive aggregate, general provenance/automatic why-not, 전체 WCOJ,
adaptive/ML cost model, full Cascades/equality saturation·Differential clone은
제외한다. Required flow witness를 먼저 검증한다. [Soufflé provenance](https://souffle-lang.github.io/provenance)는
bounded evidence의 참고이며 신규 dependency를 선택하지 않는다.

원래 RFC와 제안 문법 예시는 [ADR-0001의 복구 기준](0001-frontend-campaign-consolidation.md#2026-10-07-final-rfc-and-ticket-pass)에서
복구한다. 실행 ID·소유자·의존성은 remaining.md 한 곳에서 관리한다.
