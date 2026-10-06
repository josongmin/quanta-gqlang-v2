# 에이전트 전달 프롬프트: Quanta Index–Semantica 통합 캠페인 감사·설계

작성: 2026-10-04. 아래 본문 전체를 통합 캠페인 에이전트에 전달한다.
이 문서는 작업 지시와 이전 관찰의 인수 자료다. 구현 완료·현행 상태·qualification을 증명하는 RFC가 아니다.

## 통합 캠페인 산출물 위치

현행 소스 감사·통합 RFC·레포별 액션·검증은 Semantica의 [authoritative 인덱스](../../../semantica-codegraph-v2/docs/plans/oct-4-index-semantica-integration/README.md)에 있다. 이 파일은 인수 지시로 유지한다. 이번 통합 계획의 QGLang runtime 변경은 0이며, 일반 typed join/V4 logical·result cutover는 [DSL 주 캠페인](../rfcs/2026-10-04-code-query-dsl-and-substrate.md)이 소유한다. Index 없는 실제 두 fact join 설치 milestone의 선행 조건에 Index를 추가하지 않는다.

---

## 1. 임무와 최종 목적

너는 `quanta-index`, `semantica-codegraph-v2`, `quanta-gqlang-v2`의 현재 소스와 실제 호출 경로를 조사하고, **Quanta Index를 Semantica의 선택적 검색/후보 접근 경로로 통합하는 독립 캠페인**의 최종 감사·RFC·구현 액션 리스트를 작성한다.

핵심 질문은 다음과 같다.

1. 세 레포는 지금 실제로 어떻게 연결돼 있는가? 검색 제품 경로와 QGLang 분석 질의 경로는 어디서 만나고 어디서 끊기는가?
2. 기존 계약·authority·publication·generation·lease 중 무엇을 그대로 재사용할 수 있는가?
3. 검색 candidate를 같은 source의 canonical analysis fact/entity에 연결하려면 어느 owner를 수정해야 하는가?
4. 어떤 검색은 독립적인 ranked 결과이고, 어떤 검색은 exact 분석의 합법적인 접근 최적화인가?
5. 현재 준비된 코드, 연결되지 않은 코드, 명백한 계약 결함, 실행 증거 부족을 어떻게 구별할 것인가?
6. 최소 구현과 후속 확장을 나누면 각 레포의 변경, 의존성, 실패 처리, 검증 gate는 무엇인가?

이번 작업의 산출물은 **현재 소스 기반 감사와 구현 가능한 계획**이다. 이 지시만으로 대규모 product code 변경·commit·push·배포를 진행하지 않는다. 문서 작성·정리와 안전한 read-only 조사, 필요한 최소 검증은 진행한다. 작은 probe가 필요하면 제품 코드와 분리하고 목적·범위를 기록한다.

계획의 목적은 특정 라이브러리를 넣거나 proxy를 만드는 것이 아니다. Source가 일치하는 검색·분석 실행, 정확한 결과 의미, bounded resource 사용, 추적 가능한 실패를 확보하는 것이다. 새로운 추상화는 기존 owner로 해결할 수 없는 구체적인 경계를 확인한 경우에만 제안한다.

## 2. 작업 레포와 작업 규칙

작업 시작 레포는 `/Users/songmin/Documents/code-new/semantica-codegraph-v2`로 한다. 아래 세 레포를 직접 조사하며, 다른 레포의 docs 링크는 실제 파일 위치를 기준으로 해석한다.

| 레포 | 절대 경로 | 예상 소유권 — 현재 소스로 재검증 |
| --- | --- | --- |
| QGLang | `/Users/songmin/Documents/code-new/quanta-gqlang-v2` | 문법/HIR/sema/normalized logical IR, frontend-contract, CLI·진단 |
| Semantica | `/Users/songmin/Documents/code-new/semantica-codegraph-v2` | 실행 admission, 분석 source/authority, fact·graph·flow owner, planner/executor, SDK/daemon, Index consumer adapter |
| Quanta Index | `/Users/songmin/Documents/code-new/quanta-index` | 검색 corpus/publication, generation·readiness, query plane, candidate·window/cursor, SDK/IPC |

- 각 레포의 적용되는 `AGENTS.md`부터 읽는다. 문서·코드 owner 지시와 검증 명령을 따른다.
- 각 레포의 branch/HEAD/status, relevant dirty paths, worktree와 의존성 pin을 먼저 기록한다. Shared checkout에 다른 작업이 있을 수 있다. Unrelated dirty changes를 정리하거나 덮어쓰지 않는다.
- 관찰 중 HEAD가 움직이면 named source의 변경 여부를 확인한다. 다른 HEAD/dirty source의 receipt를 하나의 제품 증거로 합치지 않는다.
- `rg`/`rg --files`로 task-named 파일·symbol부터 찾는다. 모든 docs나 대형 plan packet을 먼저 읽지 않는다.
- 이름만 있는 trait/DTO/mock/test-support를 live production caller와 동일시하지 않는다. Caller→validation→owner→transport→producer→result를 추적한다.
- 브레이킹체인지는 가능하다. 다만 version/digest/codec/vector/pin/SDK/CLI migration과 mixed-version rejection 비용을 반드시 포함한다.
- 개발자는 혼자이며 비용을 줄여야 한다. 검증은 로컬 중심이다. Hosted CI·클라우드 서비스·새 daemon·proxy·추가 레포를 선행 조건으로 넣지 않는다.
- Rust/product 검증은 해당 owner의 build discipline을 따른다. Semantica가 QBC를 요구하면 그 경로를 사용한다. 큰 빌드 전에 가장 작은 의미 있는 owner-local rail을 선택한다.
- 런타임 테스트·E2E·성능 비교를 실행하지 못했으면 정확히 `NOT_RUN`과 이유를 남긴다. 정적 확인·compile·unit·설치 E2E의 증거 수준을 구분한다.

## 3. 배경: 기존 DSL 캠페인과의 관계

QGLang/분석 실행의 별도 주 캠페인은 다음을 목표로 한다.

- Host-agnostic typed logical V4: 전체 rule head/body, typed variable, occurrence, binding/correlation, SCC/strata 보존.
- `relation`과 기본 `query`는 set, 명시적 `rows query`는 bag. Projection/union/재귀의 dedup boundary를 고정.
- Semantica 실제 fact를 typed join하고, 유한 재귀와 model 기반 flow/CHECK로 확장.
- Result/Completion에서 Complete·Incomplete·Unsupported·Failed를 구분. `0 rows`를 전체 부재·안전함으로 자동 해석하지 않음.
- Flow profile/model/summary coverage와 required witness를 검증.
- 정확한 batch 실행부터 시작하고 optimizer·signed delta 유지 실행은 별도 gate 이후.

**주 캠페인의 첫 milestone은 Index 없이 실제 두 fact를 join해 설치 프로세스에서 typed row를 반환하는 것이다.** Index 통합을 그 milestone의 필수 선행 조건으로 만들지 않는다.

이번 통합 캠페인은 기존 액션의 **A01-I: analysis source↔search corpus 발행 관계**와 **A08-I: candidate→canonical fact adapter**를 구체화한다. Source identity·nominal key·coverage·completion의 공통 계약만 주 캠페인과 맞춘다. V4가 아직 제안 단계라면 그 상태를 그대로 기록하고, 현재 가능한 검색 감사·publication 개선과 V4에 의존하는 분석 join 단계를 나눈다.

먼저 필요한 부분만 읽을 현행 설계 문서:

- [DSL RFC](../rfcs/2026-10-04-code-query-dsl-and-substrate.md)
- [DSL 액션 리스트](../rfcs/2026-10-04-code-query-action-list.md)
- [소스 감사 인덱스](./2026-10-04-reference-source-review.md)
- [기존 V3 계약 수리 owner packet](./2026-10-04-frontend-consumer-contract-plan.md): 관련 section만 읽고 기존 상태·receipt를 덮어쓰지 않는다.

기본 구조 제안은 `QGLang logical program → Semantica admission/planner → fact providers + optional Index adapter`다. Index는 검색을 실행하고 Semantica는 source 결합, candidate 해석, 후속 join·분석·최종 결과를 책임진다. 이는 선호하는 설계 출발점이지 검증 면제 사유가 아니다.

## 4. 인수 시점의 관찰 — 확정 사실로 복사하지 말 것

2026-10-04 이전 조사와 직전 재확인에서는 다음을 관찰했다. 에이전트가 시작할 때 반드시 현재 source를 다시 확인한다.

- QGLang 관찰 HEAD: `9060e46997dd9c62080a18bdfae06f8c162442e2` + 다수 dirty 변경. V4 RFC/액션은 제안 문서이며 untracked였다.
- Semantica 검색 재감사 HEAD: `a179cc80324327a95d98d9910438ea57912db6ba`. 인수 프롬프트 작성 중 HEAD가 `10cec7c11f45417a0d438322beb0d26e4da2cbd6`로 이동했다. 아래 주요 검색·relation 파일의 status는 clean이었지만 이 두 SHA를 하나의 실행 receipt로 취급하지 않는다.
- Index 관찰 HEAD: `e68db574204a1db71263a30240d3d374e8330dcd`. 확인한 query response 파일은 clean이었다.
- Semantica에는 registered search handler→runtime Index ingress→Index 호출이 있다. Generation capture, pinned request, response/candidate generation 검증도 있다.
- 제품 SDK 검색의 `published-indexing-authority-runtime` 경로에는 product-visible generation/snapshot 검증, owner-issued source authority, durable Acked product scope, aggregate reader lease 연결이 있다. 실제 설치 profile이 이 feature를 사용하는지와 lease가 끝까지 유지되는지는 다시 확인한다.
- `capture_registered_search_index_v1`은 session의 retained source authority를 요구한다. Authority 경로의 capture는 `Some(required_generation)`를 전달한다.
- 별도 direct helper `execute_registered_search_concrete_v1`은 `capture_active_v1(..., None)`을 호출했다. 이것만 보고 모든 제품 검색이 unpinned라고 주장하지 않는다. Helper의 caller·용도·도달 가능성을 따로 조사한다.
- Index의 `TextQueryResponse`에는 generation, rank_unit, results, `QueryResultWindowV2`, next_cursor가 있다. 이들 필드 존재만으로 분석 입력의 exhaustiveness가 증명되는 것은 아니다.
- 관찰한 QGLang V3 relation admission은 단일 `RepoPath` schema와 제한된 연산자 subset이다. 검색 caller가 존재한다고 일반 typed fact↔candidate join까지 완료됐다고 해석하지 않는다.
- QGLang graph/flow/CHECK와 registered search는 각각 기존 경로가 있다. 일반화된 새 DSL 전체 기능·통합 completion·설치 E2E는 별도 증거가 필요하다.
- 직전 확인은 정적 조사였고 product build/test/E2E는 실행하지 않았다.

이전 문서에 “source authority가 없다”, “검색 연결이 없다”, “CHECK가 항상 미연결” 같은 문구가 있으면 현재 경로와 대조해 stale 여부를 판단한다. 반대로 코드가 있다는 이유로 검증 완료로 승격하지 않는다.

## 5. 초기 소스 탐색 지점

아래 경로는 탐색 seed다. 파일 이동·기능 분할·실제 caller를 `rg`로 찾고 현재 위치로 갱신한다. Semantica 경로는 해당 레포 기준이다.

| 영역 | 탐색 seed |
| --- | --- |
| SDK registered search | `packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/query_handlers/search_handler.rs` |
| Daemon/product query ownership | `packages/analysis/quanta-v2/crates/quanta-sdk-runtime-executor/src/client/daemon_search_execution.rs`, `daemon_search_execution_aggregate_owner_v2.rs` |
| Session capture caller | `packages/analysis/quanta-v2/crates/quanta-runtime/src/sdk/code/mod.rs`: `capture_registered_search_index_v1` |
| Retained source authority | `packages/analysis/quanta-v2/crates/quanta-runtime/src/sdk/search_builder/index_owner_env_authority.rs`: `request_context_v1`, `capture_search_index_v1` |
| Existing search execution | `packages/analysis/quanta-v2/crates/quanta-runtime/src/sdk/search_builder/index_owner.rs`, 관련 window/cursor mapping |
| Generation-bound ingress | `packages/analysis/quanta-v2/crates/quanta-runtime-retrieval-kernel/src/index_sdk_ingress/generation_bound_search_index_v1.rs`, `connect`, `publish` |
| Search plan/result mapping | `quanta-sdk-query-backend-owner`의 registered-search plan/core/outcome 및 public envelope mapping |
| Product publication/readiness | `quanta-contract-retrieval`의 `SearchOwnerAggregateQueryRequestV2`, aggregate host/closeout/reader lease owner |
| QGLang admission/execution | `quanta-runtime-public-ingress-surface`, `quanta-gqlang-v2-exec-contract`, `quanta-gqlang-v2-exec-host-spi`, `quanta-runtime-execution/src/gqlang` |
| Current relation restriction | `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/relation_plan_v3.rs`: `single_path`, `compile_relation_plan_v3` |
| Physical bridge | `packages/analysis/quanta-v2/crates/quanta-runtime-execution/src/gqlang/physical_plan_v1`, 기존 executor/planner |

Index의 초기 탐색 지점:

- `crates/quanta-index-contract/src/results/query_responses.rs`: candidate/source/generation/rank/window/cursor, decoder validation.
- `crates/quanta-index-contract-base/src/results/query_window.rs`: completeness/window의 실제 owner와 생성 조건.
- `crates/quanta-index-contract/src/query`: query constraints, generation selector, cursor의 query/order/epoch binding.
- `crates/quanta-index-sdk`, `quanta-index-ipc`, `quanta-index-searchd*`: transport, deadline, retry, cancellation, message size, installed service path.
- `crates/quanta-index-catalog`, `quanta-index-search-plane`, `quanta-index-lexical`, `quanta-index-semantic`: publication, active/retained generation, exactness, cursor lifetime, sparse/dense readiness.
- `SourceFileRevision`, corpus identity, candidate identity의 정의와 실제 producer. Candidate의 optional source가 어떤 route에서 항상/일부/전혀 제공되는지 확인.
- Semantica가 어떤 SDK/contract version·path/git pin을 소비하는지 Cargo manifests/lock에서 확인. 실제 local source와 소비 bytes가 같은지 확인.

QGLang은 `frontend-contract`, pipeline execution lowering, language ownership, public execution V3 문서를 필요한 만큼 확인한다. Runtime Index 계약을 frontend로 옮기는 제안을 자동으로 하지 않는다.

## 6. 감사 A: 현재 architecture와 reachable execution

다음 경로를 각각 별도 call graph로 작성한다. 제안 구조와 현재 구조를 한 그림에 섞지 않는다.

1. Public SDK/daemon의 lexical/symbol/semantic/hybrid/canonical 검색.
2. Direct registered-search helper와 그 production/test/internal caller.
3. Source 변경→분석 fact publication→검색 corpus publication→readiness/closeout→query admission.
4. QGLang source→compiled artifact→consumer validation→source admission→physical execution→SDK result.
5. Search candidate→fact/entity resolution→join/graph/flow가 이미 있으면 그 실경로. 없으면 끊긴 연결을 명시.

각 경로에서 함수/타입/feature/production caller, source·generation 발행 owner, IPC 경계, 실패 mapping, 결과 identity, lease acquisition/release를 표로 기록한다. 구현됨·production reachable·focused tested·installed qualified를 따로 표시한다.

기존 구현을 다음으로 나눠라: 그대로 재사용 / 배선만 필요 / 계약 보강 필요 / 구현 누락 / 증거 부족. 정적 결함과 proof gap을 동일한 P0로 묶지 않는다. 확정 결함에는 입력·faulty path·기대 동작·사용자 영향·독립 회귀를 붙인다.

## 7. 감사 B: source·corpus·generation의 공동 authority

단일 “snapshot ID”라는 말로 다른 owner의 identity를 합치지 말고 다음 축을 조사한다.

- Workspace/root/repo/incarnation/tenant 또는 접근 권한의 경계.
- Analysis snapshot/source closure·file revision/content digest·overlay·의존성 revision.
- Search corpus identity, lexical manifest/generation/digest, semantic model/index generation, composite active identity/readiness.
- Product-visible serving generation과 durable publication/closeout/Acked 상태.
- Schema/contract/query/profile/model/normalization version과 호환 범위.

각 축의 issuer, 검증 위치, 저장/transport 타입, lifetime, forged/stale/mismatch 거절을 조사한다. 분석 source와 검색 corpus가 일치한다는 관계가 이미 owner-issued 계약으로 표현되는지 확인한다. 있다면 연결을 재사용하고, 없으면 무엇이 빠졌는지 정확히 정의한다. Raw path/repo name/revision 문자열의 우연한 일치를 공동 authority로 대체하지 않는다.

특히 조사할 전이: unsaved overlay, file delete/rename, repo recreation, dependency 변화, sparse ready/dense pending, publication crash/restart, active head switch, retained generation eviction, model rebuild. 이전 source가 계속 유효한 경우 기존 snapshot을 유지할 수 있는지, 현재 capture가 active를 요구해 거절하는지, pinned retained open이 가능한지 구분한다.

추가 binding DTO가 필요하면 먼저 기존 aggregate/source/corpus identity의 수용 가능성을 확인한다. 필요 필드·issuer·validator·version·expiry만 정의하고 owner를 중복 발명하지 않는다. Typed record 자체가 신뢰 증명이 아니므로 외부 bytes의 issuer 검증도 명시한다.

## 8. 감사 C: candidate와 canonical fact/entity identity

다음 identity를 분리한다: lexical chunk, ranked file, declaration symbol, analysis file/entity/node/edge, finding. `candidate_id`·path·line range를 canonical semantic entity ID와 동일시하지 않는다.

- Mapping cardinality가 1:1/1:N/N:1 중 무엇인지, chunk duplicate와 projection dedup을 어느 boundary에서 처리하는지 조사한다.
- 같은 source의 file revision/content와 source-qualified nominal entity key로 검증 가능한가? 범위·위치만 있는 candidate는 어떻게 resolve하는가?
- Same file의 overload/nested declaration·동일 이름·cross-language symbol, 다중 repo path 충돌, rename/rebuild/overlay로 위치가 변하는 경우를 다룬다.
- Missing/deleted/ambiguous/foreign source candidate의 처리와 budget을 정한다. 단순 누락해도 되는 ranked query와 exact 분석에서 completeness를 깨뜨리는 경우를 구분한다.
- Source/generation-bound 내부 ID와 cross-generation stable semantic key의 용도를 구분한다. 안정성을 증명하지 않은 finding/entity key를 재사용하지 않는다.
- Source 접근 권한 검사 전에 candidate snippet/location이 결과나 trace로 노출되지 않도록 실제 validation 순서를 확인한다.

Mapping의 issuer와 validator는 현재 canonical fact owner에 둔다. Index에 분석 solver를 넣거나 path 문자열 heuristic으로 “연결 성공”을 만드는 계획은 제안하지 않는다.

## 9. 감사 D: 검색 결과 의미와 분석의 합법적 사용

아래 네 가지를 반드시 구분해 지원 행렬을 작성한다.

| 사용 방식 | 필요한 의미 |
| --- | --- |
| 명시적 ranked/top-k 검색 질의 | 선언한 ranking/window 안의 결과. 이 질의의 완료는 전체 corpus 부재 증명이 아님 |
| Exact predicate의 Index 접근 최적화 | 실제 predicate와 동치 또는 누락 없는 superset + exact residual; 전체 입력 필요 시 exhaustive enumeration |
| 사용자가 검색 결과로 scope를 정한 분석 | 검색 seed와 그 밖의 scope/traversal 권한을 명시. Findings는 그 scope에 한정하며 repository 전체 clean으로 확대 불가 |
| Count/NOT EXISTS/anti-join/전체 finding scan | 필요한 입력 범위의 exhaustive coverage와 해당 정책의 부재 권한. Approximate/capped prefix로 대체 불가 |

Text normalization/tokenization/case/prefix/infix/regex/fuzzy·semantic similarity가 exact analysis predicate와 같은지 실제 source로 확인한다. False positive는 residual로 제거할 수 있으나 false negative는 residual로 복구할 수 없다. “검색 순위가 좋아 보임”은 superset의 증명이 아니다.

특히 검사할 항목:

- `rank_unit`, score/tie-order, page-local limit vs logical global limit/offset, duplicate chunk→file/entity projection.
- Residual이 앞 후보를 제거하면 필요한 결과 수를 채우기 위해 추가 page를 읽는가? 필요한 exhausted/continuation 상태를 잃지 않는가?
- Query/filter/order/corpus가 바뀐 cursor와 expired cursor를 거절하는가? Producer window와 token의 관계를 decoder도 검증하는가?
- Dense approximate/hybrid fusion이 exhaustive exact 후보를 보장하는가? 그렇지 않으면 정확성 contract와 분리되는가?
- Endpoint filter, 중간 graph traversal domain, 접근 권한을 분리한다. Entry 후보를 좁혔다고 traversal을 그 후보의 induced subgraph로 바꾸지 않는다.
- Set/bag·nominal key·correlation·null/Option 의미를 adapter와 후속 join이 보존하는가?

Index 사용 중 오류/expiry/budget/cancel 이후 다른 active corpus로 재검색해 성공시키지 않는다. 같은 admitted source에서 의미가 동일한 baseline fallback이 가능한 경우만 정책·비용·실패 mapping을 정의한다. Fallback 자체를 신규 source admission 없이 silent retarget으로 사용하지 않는다.

## 10. 감사 E: completion·resource·transport·lifecycle

Frontend logical bytes에 runtime Index handle이나 manifest pin을 넣지 않는다. Runtime admission/execution binding이 logical query, analysis source, Index corpus, profile/model, request budget을 결합한다. 기존 V3 결과와 제안 V4 Result/Completion은 구현 상태를 분리한다.

- Provider coverage, 검색 window completeness, logical query completion, 전체 부재 권한은 다른 개념이다. 각각 issuer·검증·SDK 표현을 정한다.
- Authored limit을 의미대로 완료한 ranked query와 host cap으로 끊긴 exact query를 구분한다. Progress/page는 terminal Complete가 아니다.
- Result mapping이 Index mismatch/not-ready/expiry/partial/cap/worker error를 0행 또는 clean으로 바꾸는지 추적한다.
- Source/model/profile가 다른 page·witness·output reference를 합치지 않는다. Finding identity와 ranked candidate identity를 분리한다.
- Deadline을 connect/capture/IPC/search/pagination/resolve/join/serialization까지 전파하는지 확인한다. 단계마다 full budget을 재부여하지 않는다.
- Retained candidate·dedup·join/sort·cursor/evidence memory를 owner reservation에 연결한다. IPC payload/response bytes와 batch/fanout/repeated work를 제한한다.
- Cancellation·disconnect·drop·retry·restart에서 worker task·reader lease·cursor history·reservation이 언제 해제되는지 확인한다.
- Retained source/corpus가 query/evidence/cursor 수명 동안 살아 있는가? Compaction/eviction 압력 시 거절·명시적 expiry인지 조사한다. 오래된 결과를 새 corpus로 이어붙이지 않는다.
- Sparse/dense/composite publication의 일관성, update/delete replay safety, schema/model migration 시 cache invalidation을 확인한다. Index 자체의 증분 검색과 Semantica 분석의 증분 유지 실행을 혼동하지 않는다.

무조건 transaction/새 protocol/분산 coordinator를 넣지 않는다. Existing publication/lease가 해결하는 범위와 실제 누락 구간을 먼저 증명한다.

## 11. 설계 결정과 before/after

최종 RFC는 최소 세 대안을 같은 기준으로 비교한다.

1. **Semantica 내부 owner adapter 재사용/확장**: 기존 ingress/source authority/publication/lease에 연결.
2. **Semantica 안의 얇은 provider façade/module 분리**: 기존 owner 간 compile-time 경계를 정리할 구체적 필요가 있는 경우.
3. **독립 façade/proxy/service/repo**: 실질적인 다중 consumer·독립 lifecycle·권한/운영 경계가 있는지 반증 관점으로 평가. 현재는 추가 네트워크 hop·failure domain·배포 비용 때문에 채택 근거가 약하다는 출발점.

원하는 결론에 맞춰 대안을 평가하지 않는다. 기존 경계의 문제를 source로 입증하고 ownership, identity/authority, semantics, operational failure, dependency direction, solo maintenance, migration, verification 비용으로 판단한다. Runtime을 QGLang에 넣거나 별도 generic query engine으로 바꾸는 선택은 대표 질의의 근거 없이 제안하지 않는다.

다음 시나리오의 before/after를 실제 함수·타입·분기 변화와 결과로 설명한다.

- 기존 lexical/symbol 검색을 실행하는 사용자.
- 선택된 검색 scope의 file/entity에 분석 fact를 join하는 사용자.
- Index가 exact predicate의 접근 경로를 가속하는 질의.
- 검색된 HTTP entry에서 corpus 외부 후보가 아닌, 허용된 analysis domain의 중간 node를 거쳐 SQL target에 도달하는 질의.
- semantic/hybrid top-k 후보를 명시적으로 탐색하는 사용자.
- 전체 count/absence/clean을 요청하는 사용자.
- overlay/source mismatch, active switch, expiry, partial dense publication, timeout/cancel/restart.

각 시나리오에 expected row/finding/completion, Index가 없거나 사용할 수 없을 때의 동작, 실행 가능한 current path와 proposed path를 분리한다. Syntax가 미구현이면 예시는 pseudo DSL이라고 표시한다.

## 12. 검증 계획과 독립 oracle

Implementation gate를 기계적으로 통과시킬 mock만 설계하지 않는다. 잘못된 동작을 실제로 구별하는 독립 기대값과 positive/negative 회귀를 정의한다.

| 회귀군 | 독립적으로 확인할 내용 |
| --- | --- |
| Source/corpus authority | 동일 repo/revision 문자열이나 다른 content/overlay/incarnation인 corpus 거절; 정상 공동 발행은 수용 |
| Generation | capture 후 active switch; valid retained read와 unavailable retained read 구분; mixed candidate/page 거절 |
| Entity mapping | overload/nested/동일 path 다른 repo·동일 파일 여러 chunk·delete/rename·ambiguous mapping; 다른 entity로 연결되지 않음 |
| Exact 후보 | corpus 직접 predicate oracle 대비 index on/off 동등성; residual false positive 제거와 false negative 검출 |
| Pagination/limit | 필요한 match가 첫 page 밖에 존재, duplicate projection, residual 후 refill, last-page exhaustiveness, wrong-query/expired cursor |
| Negation/count | capped/partial/approximate 후보가 NOT EXISTS·전체 count·clean 성공으로 바뀌지 않음 |
| Ranked scope | top-k scope의 분석 완료와 repository 전체 완료를 구분; scope 밖 traversal의 허용·거절 |
| Resource/lifecycle | reserve 실패·cap·cancel/disconnect·daemon restart; 작업/lease/reservation 해제, partial answer의 terminal 상태 |
| Migration | incompatible bytes/schema/profile/model/pin 거절; codec/vector/golden의 producer–consumer 일치 |

검증 단계는 contract/decoder → owner adapter → runtime integration → 설치 SDK/daemon/실제 Index process 순서로 잡는다. 가능한 smallest rail부터 실행한다. Installed proof를 실행하지 않았으면 “가능하다”는 계획과 “통과했다”는 증거를 나눈다.

성능은 correctness 통과 후 같은 corpus/source/query/policy에서 비교한다. Index 없이/있이 cold/warm, publication/index build 비용, planning/IPC/resolve/join/query latency, scanned candidates/pages, charged work, peak RSS/retained memory, result completeness를 함께 기록한다. App 단위 latency에는 전체 비용을 포함한다. 고정한 workload의 개선만 주장하고 이미 있는 검색 benchmark를 분석 join 성능의 증거로 재사용하지 않는다.

## 13. 단계별 계획 요구사항

최종 액션은 각 row에 ID, 목적, defect 또는 capability gap, owning repo/module, current reusable code, proposed contract/code change, prerequisite, compatibility, independent oracle, verification command, acceptance condition, risk/stop condition을 포함한다. 대형 단계는 리뷰 가능한 배치로 나누되 line-count-only 쪼개기를 하지 않는다.

초안 단계 구조는 다음과 같다. 감사 결과에 따라 합치거나 수정하되 이유를 설명한다.

1. **I00 현재 상태·경계 확정:** live caller·feature·pin·source 상태와 공통 fixture/oracle.
2. **I01 source↔corpus authority:** existing product search binding을 재사용하고 analysis fact closure와 연결할 최소 contract 확정.
3. **I02 candidate identity/coverage:** canonical resolution과 exact/ranked/scope 계약, window/cursor/result mapping.
4. **I03 최소 adapter:** 가장 단순하고 현재 owner가 지원하는 exact 접근 또는 명시적 ranked-scope 분석 하나. V4 blocker면 그 의존성을 분리하고 검색 owner 준비를 진행.
5. **I04 installed integration proof:** source-valid positive와 mismatch/partial/expiry/resource negative를 실제 SDK/daemon/Index process에서 확인.
6. **I05 planner 선택·측정:** admitted baseline과 legal Index access 후보, supported pushdown/residual, deterministic fallback. 선택 이유와 실제 비용 관찰.
7. **I06 후속 확대:** multi-provider/multi-repo, overlay, dense/hybrid, flow 입력, cache/증분의 실제 수요에 따른 확장. 기본 기능의 필수 선행 조건으로 모두 넣지 않음.

기존 A01-I/A08-I 및 주 캠페인 A01-F/A03/A05/A06 등과의 대응을 명시한다. Index-only 개선, Semantica 소비자 개선, 공동 contract 변경, QGLang semantic 변경을 구분한다. QGLang 변경이 필요하지 않으면 없다고 명확히 쓰고 수량 맞추려고 작업을 만들지 않는다.

## 14. 레퍼런스·라이브러리 판단

Reference가 필요하면 공식 문서와 실제 primary source를 사용한다. 먼저 이미 clone된 source를 활용하고 pin을 기록한다.

- 감사 인덱스: `quanta-gqlang-v2/docs/audits/2026-10-04-reference-source-review.md`.
- Clone manifest: `/Users/songmin/Documents/code-new/.reference-audits/2026-10-04-code-query/manifest.json`.
- DataFusion: provider pushdown exact/inexact/unsupported, residual/limit, streaming/memory/cancel.
- Substrait: required semantics, producer–consumer subset와 version/profile 검증.
- CodeQL/Joern: 검색 seed와 분석 scope/model/flow witness의 차이. 검색 rank를 solver correctness로 대체하지 않음.

로컬 기존 SDK/contract/owner adapter를 먼저 조사한다. 새 dependency는 실제 missing capability, Rust/MSRV/license/runtime/build 비용, source/authority/resource contract 적합성, 작은 동일 workload probe로 필요성을 증명한 경우만 후속 후보로 둔다. “SOTA”라는 말로 신규 엔진·storage·e-graph·SMT를 정당화하지 않는다.

## 15. 최종 산출물과 완료 조건

문서 위치는 Semantica의 적용되는 docs convention과 기존 통합 campaign을 먼저 찾아 결정한다. 같은 내용을 세 레포에 복제하지 않는다. Semantica를 주 consumer owner로 감사/RFC/액션의 authoritative 문서를 두고, Index와 QGLang에는 필요한 cross-link/소유권만 둔다. 이미 동등한 현행 문서가 있으면 갱신하고 stale packet을 늘리지 않는다.

최종 산출물:

1. **현재 통합 감사:** exact source tuple, reachable caller/owner/feature/pin, call graphs, status matrix, 확정 defect와 proof gap.
2. **통합 RFC:** 목적/배경/기존 캠페인과의 경계, 선택 구조·대안의 기각 이유, contract/authority/identity/coverage/completion/resource/lifecycle, before/after, migration.
3. **레포별 액션 리스트:** 위의 owner·변경·선행·독립 oracle·command·exit/stop condition을 포함. 지금 필요한 작업과 후속 확장을 분리.
4. **검증 기록:** 실제 실행한 command/결과/source와 `NOT_RUN` 항목. Test 파일의 존재와 해당 test 실행 결과를 구분.
5. **의존성 도식:** DSL 주 캠페인을 기다리지 않고 가능한 작업, V4에 막힌 작업, 공동 계약 확정이 필요한 작업을 표시.

완료 시 답해야 할 질문:

- Semantica에서 Index를 접근하는 구조가 현재 요구에 적합한가? 어떤 근거이며 실제 대안보다 어떤 tradeoff가 있는가?
- 기존 검색 통합은 어느 수준까지 준비됐고, 분석 통합은 정확히 어느 함수/contract에서 끊기는가?
- Substrate를 먼저 보강해야 하는가, adapter/contract만 연결하면 되는가? 항목마다 답하라.
- 각 레포에서 지금 수정할 파일/owner·행동 변화·완료 증거는 무엇인가?
- QGLang 변경 없이 가능한가? 새 module/crate/proxy/repo가 필요한가? 필요한 경우만 근거를 제시하라.
- Exact query와 ranked/scope query 각각 무엇을 보장하고 무엇을 주장할 수 없는가?
- 첫 배치의 구현·검증이 구체적으로 review 가능한가? 장기 wishlist가 아닌 실행 가능한 순서인가?

보고는 한국어로 직접적이고 기술적으로 작성한다. 검증된 사실, 설계 제안, 추론, 미확인을 구분한다. 레퍼런스의 기능·명성으로 현지 구현의 성숙도를 평가하지 않는다. 치명적인 문제만 과장해 나열하지 말고 이미 있는 구현도 보존한다. 설계는 기존 owner의 책임을 명확히 하고, source가 일치하는 실제 질의를 끝까지 실행하는 데 필요한 변경에 집중한다.
